use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, ExtendedId, StandardId};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;

use super::enums::{BusType, CanInterface};
use super::messages::{CanMessageTrait, RawCanMessage};

use log;

 const BAUDRATE: u32 = 1_000_000;

#[derive(Debug)]
enum Command {
    Send(CanFrame),
    Shutdown,
}

pub trait DynamicCanListener {
    fn on_message_received(&self, msg: &RawCanMessage);
    fn on_error(&self, exc: anyhow::Error);
    fn stop(&self);
    fn listen(&self, rx: broadcast::Receiver<RawCanMessage>) -> JoinHandle<Result<()>>;
}

pub struct CanSimpleListener<T: CanMessageTrait + Send + 'static> {
    _phantom: PhantomData<T>,
    callback: Option<Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>>,
    queue_tx: mpsc::Sender<RawCanMessage>,
    queue_rx: Mutex<mpsc::Receiver<RawCanMessage>>,
    bus_error: Mutex<Option<anyhow::Error>>,
    is_stopped: AtomicBool,
}

impl<T: CanMessageTrait + Send + 'static> CanSimpleListener<T> {
    pub fn new(_phantom: PhantomData<T>, callback: Option<Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>>) -> Self {
        let (queue_tx, queue_rx) = mpsc::channel(32);
        Self {
            _phantom,
            callback,
            queue_tx,
            queue_rx: Mutex::new(queue_rx),
            bus_error: Mutex::new(None),
            is_stopped: AtomicBool::new(false),
        }
    }

    pub async fn get_message(&self) -> T {
        let mut rx = self.queue_rx.lock().await;
        let raw = rx.recv().await.unwrap();
        T::from_can_message(raw)
    }

    pub async fn wait_for_message(&self, duration: Duration) -> Option<T> {
        let fut = self.get_message();
        time::timeout(duration, fut).await.ok()
    }
}

impl<T: CanMessageTrait + Send + 'static> DynamicCanListener for CanSimpleListener<T> {
    fn on_message_received(&self, msg: &RawCanMessage) {
        if T::matches(msg) {
            let _ = self.queue_tx.blocking_send(msg.clone());
        }
    }

    fn on_error(&self, exc: anyhow::Error) {
        *self.bus_error.blocking_lock() = Some(exc);
    }

    fn stop(&self) {
        self.is_stopped.store(true, Ordering::Relaxed);
    }

    fn listen(&self, mut rx: broadcast::Receiver<RawCanMessage>) -> JoinHandle<Result<()>> {
        let self_arc = Arc::new(self.clone()); // if Clone impl, but for simplicity assume
        tokio::spawn(async move {
            while let None = *self_arc.bus_error.lock().await {
                if self_arc.is_stopped.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(msg) = time::timeout(Duration::from_millis(10), self_arc.get_message()).await {
                    if let Some(cb) = &self_arc.callback {
                        (cb)(msg).await;
                    }
                }
            }
            if let Some(err) = self_arc.bus_error.lock().await.take() {
                Err(err)
            } else {
                Ok(())
            }
        })
    }
}

pub struct CanSimple {
    command_tx: mpsc::Sender<Command>,
    broadcast_tx: broadcast::Sender<RawCanMessage>,
    join_handle: JoinHandle<()>,
    listeners: Arc<std::sync::Mutex<Vec<Arc<dyn DynamicCanListener + Send + Sync>>>>,
}

impl CanSimple {
    pub fn new(can_interface: CanInterface, bustype: BusType) -> Self {
        let channel = can_interface.value();
        let (command_tx, mut command_rx) = mpsc::channel(32);
        let (broadcast_tx, _) = broadcast::channel(256);
        let listeners = Arc::new(StdMutex::new(Vec::new()));
        let join_handle = tokio::task::spawn_blocking({
            let broadcast_tx = broadcast_tx.clone();
            let listeners = listeners.clone();
            move || {
                let mut cs = CanSocket::open(channel).expect("Failed to open CAN socket");
                // Flush bus
                while cs.recv_timeout(Duration::ZERO).is_ok() {}
                loop {
                    let frame_res = cs.recv_timeout(Duration::from_millis(10));
                    match frame_res {
                        Ok(frame) => {
                            let raw = Self::frame_to_raw(&frame);
                            let _ = broadcast_tx.send(raw);
                        }
                        Err(socketcan::Error::Timeout) => {},
                        Err(e) => {
                            let g = listeners.lock().unwrap();
                            for l in &*g {
                                l.on_error(anyhow!(e));
                            }
                            break;
                        }
                    }
                    while let Some(cmd) = command_rx.try_recv() {
                        match cmd {
                            Command::Send(f) => {
                                if let Err(e) = cs.write(&f) {
                                    log::error!("Error sending frame: {}", e);
                                }
                            }
                            Command::Shutdown => return,
                        }
                    }
                }
            }
        });
        Self {
            command_tx,
            broadcast_tx,
            join_handle,
            listeners,
        }
    }

    pub fn register_callbacks<T: CanMessageTrait + Send + 'static>(&self, msg_cls_callbacks: Vec<(PhantomData<T>, Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>)>) {
        let mut g = self.listeners.lock().unwrap();
        for (phantom, callback) in msg_cls_callbacks {
            let callback = Box::new(move |m| Box::pin(callback(m)));
            let listener = Arc::new(CanSimpleListener::new(PhantomData::<T>, Some(callback)));
            g.push(listener);
        }
    }

    pub async fn send(&self, msg: impl CanMessageTrait) -> Result<()> {
        let raw = msg.as_can_message();
        let id = if raw.is_extended_id {
            ExtendedId::new(raw.arbitration_id).ok_or(anyhow!("Invalid extended ID"))?
        } else {
            StandardId::new(raw.arbitration_id as u16).ok_or(anyhow!("Invalid standard ID"))?
        };
        let frame = CanFrame::new(id, &raw.data, false, false).unwrap();
        self.command_tx.send(Command::Send(frame)).await?;
        Ok(())
    }

    pub async fn listen(&self) -> Result<() > {
        let listeners = {
            let g = self.listeners.lock().unwrap();
            g.clone()
        };
        let mut tasks = vec![];
        for l in listeners {
            let rx = self.broadcast_tx.subscribe();
            tasks.push(l.listen(rx));
        }
        for task in tasks {
            task.await??;
        }
        Ok(())
    }

    pub async fn shutdown(self) {
        let g = self.listeners.lock().unwrap();
        for l in &*g {
            l.stop();
        }
        let _ = self.command_tx.send(Command::Shutdown).await;
        let _ = self.join_handle.await;
    }

    fn frame_to_raw(frame: &CanFrame) -> RawCanMessage {
        let (arbitration_id, is_extended_id) = match frame.id() {
            socketcan::Id::Standard(id) => (id.as_raw() as u32, false),
            socketcan::Id::Extended(id) => (id.as_raw(), true),
        };
        RawCanMessage {
            arbitration_id,
            data: frame.data().to_vec(),
            is_extended_id,
        }
    }
}
