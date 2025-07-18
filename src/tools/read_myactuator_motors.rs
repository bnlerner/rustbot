extern crate havendrive;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use clap::Parser;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use havendrive::drivers::can::connection::{CanSimple, DynamicCanListener};
        use havendrive::drivers::can::enums::{BusType, CanInterface};
        use havendrive::drivers::can::myactuator_v3_msgs::{
            MotorShutdownCommand, PositionControlCommand, MyactuatorReadMotorStatus1Message, ReadMultiTurnAngleMessage,
            SpeedControlCommand, SystemBrakeReleaseCommand,
        };
        use havendrive::drivers::can::myactuator_x424_msgs::{
            QAReturnMessageType1, QAReturnMessageType2, QAReturnMessageType3, QAReturnMessageType4,
            QueryCANCommunicationIDMessage, SetCommunicationModeMessage, X424ServoPositionControlMessage,
            X424ServoSpeedControlMessage,
        };
        use havendrive::drivers::can::messages::CanMessageTrait;

        #[derive(Parser, Debug)]
        #[command(about = "Test MyActuator motors via CAN")]
        struct Args {
            #[arg(short = 'd', long)]
            discover: bool,

            #[arg(short = 'a', long)]
            test: bool,
        }

        let args = Args::parse();
        if args.discover || args.test {
            let discovered = discover_motors().await?;

            if args.test {
                for (id, motor_type) in discovered {
                    test_motor(&motor_type, id).await?;
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("This tool is only supported on Linux platforms with socketcan.");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn discover_motors() -> Result<HashMap<u32, String>> {
    let discovered = Arc::new(Mutex::new(HashMap::new()));

    let can_bus = CanSimple::new(CanInterface::Myactuator, BusType::SocketCan);

    let discovered_v3 = discovered.clone();
    let callback_v3 = Box::new(move |m: MyactuatorReadMotorStatus1Message| {
        let discovered = discovered_v3.clone();
        Box::pin(async move {
            discovered.lock().unwrap().insert(m.node_id(), "Controller V3".to_string());
            println!("Discovered Controller V3 motor with ID {}", m.node_id());
        })
    });

    let discovered_x4 = discovered.clone();
    let callback_x4 = Box::new(move |m: QueryCANCommunicationIDMessage| {
        let discovered = discovered_x4.clone();
        Box::pin(async move {
            discovered.lock().unwrap().insert(m.node_id(), "X4-24".to_string());
            println!("Discovered X4-24 motor with ID {}", m.node_id());
        })
    });

    can_bus.register_callbacks::<MyactuatorReadMotorStatus1Message>(vec![
        (std::marker::PhantomData, callback_v3),
    ]);
    can_bus.register_callbacks::<QueryCANCommunicationIDMessage>(vec![
        (std::marker::PhantomData, callback_x4),
    ]);

    println!("Scanning CAN interface can0 for motors...");

    let listen_task = tokio::spawn(can_bus.listen());

    println!("Probing for motors...");

    println!("Probing for any X4-24 motors...");
    can_bus.send(QueryCANCommunicationIDMessage::new(0)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    for node_id in 1..=7 {
        println!("Probing for V3 controller motor with ID {}", node_id);
        can_bus.send(MyactuatorReadMotorStatus1Message::new(node_id)).await?;
        sleep(Duration::from_secs_f32(0.5)).await;
    }

    listen_task.abort();
    can_bus.shutdown().await;

    let discovered = discovered.lock().unwrap().clone();
    Ok(discovered)
}

#[cfg(target_os = "linux")]
async fn test_motor(motor_type: &str, node_id: u32) -> Result<()> {
    if motor_type == "X4-24" {
        test_x4_motor(node_id).await
    } else {
        test_controller_v3_motor(node_id).await
    }
}

#[cfg(target_os = "linux")]
async fn test_x4_motor(node_id: u32) -> Result<()> {
    let can_bus = CanSimple::new(CanInterface::Myactuator, BusType::SocketCan);

    println!("Connected to CAN interface: can0");
    println!("Testing X4-24 motor with ID: {}", node_id);

    let callback1 = Box::new(move |m: QAReturnMessageType1| Box::pin(async move { println!("{:?}", m); }));
    let callback2 = Box::new(move |m: QAReturnMessageType2| Box::pin(async move { println!("{:?}", m); }));
    let callback3 = Box::new(move |m: QAReturnMessageType3| Box::pin(async move { println!("{:?}", m); }));
    let callback4 = Box::new(move |m: QAReturnMessageType4| Box::pin(async move { println!("{:?}", m); }));

    can_bus.register_callbacks::<QAReturnMessageType1>(vec![(std::marker::PhantomData, callback1)]);
    can_bus.register_callbacks::<QAReturnMessageType2>(vec![(std::marker::PhantomData, callback2)]);
    can_bus.register_callbacks::<QAReturnMessageType3>(vec![(std::marker::PhantomData, callback3)]);
    can_bus.register_callbacks::<QAReturnMessageType4>(vec![(std::marker::PhantomData, callback4)]);

    let listen_task = tokio::spawn(can_bus.listen());

    // Set to qa mode
    can_bus.send(SetCommunicationModeMessage::new(node_id, "qa".to_string())).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    println!("Testing position control (-90° → 90° → 0°)...");

    can_bus.send(X424ServoPositionControlMessage::new(node_id, -90.0, 300.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(2.5)).await;

    can_bus.send(X424ServoPositionControlMessage::new(node_id, 90.0, 300.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(2.0)).await;

    can_bus.send(X424ServoPositionControlMessage::new(node_id, 0.0, 300.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(2.0)).await;

    // println!("Testing speed control (-100 -> 0 -> 100 -> 0)...");

    can_bus.send(X424ServoSpeedControlMessage::new(node_id, -100.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(2.0)).await;

    can_bus.send(X424ServoSpeedControlMessage::new(node_id, 0.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    can_bus.send(X424ServoSpeedControlMessage::new(node_id, 100.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(2.0)).await;

    can_bus.send(X424ServoSpeedControlMessage::new(node_id, 0.0, 5.0, 0)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    listen_task.abort();
    can_bus.shutdown().await;

    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_controller_v3_motor(node_id: u32) -> Result<()> {
    let can_bus = CanSimple::new(CanInterface::Myactuator, BusType::SocketCan);

    println!("Connected to CAN interface: can0");
    println!("Testing Controller V3 motor with ID: {}", node_id);

    let callback_status = Box::new(move |m: MyactuatorReadMotorStatus1Message| Box::pin(async move {
        println!("Status: Temp={}°C, Voltage={:.1}V, Error=0x{:04x}", m.temperature, m.voltage, m.error_state);
    }));

    let callback_angle = Box::new(move |m: ReadMultiTurnAngleMessage| Box::pin(async move {
        println!("Angle: {:.2}°", m.angle);
    }));

    can_bus.register_callbacks::<MyactuatorReadMotorStatus1Message>(vec![(std::marker::PhantomData, callback_status)]);
    can_bus.register_callbacks::<ReadMultiTurnAngleMessage>(vec![(std::marker::PhantomData, callback_angle)]);

    let listen_task = tokio::spawn(can_bus.listen());

    println!("Testing Controller V3 commands...");

    println!("Testing position control (0° → 90° → 0°)...");

    can_bus.send(SystemBrakeReleaseCommand::new(node_id)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    can_bus.send(PositionControlCommand::new(node_id, 90.0, 500)).await?;
    sleep(Duration::from_secs_f32(3.5)).await;

    can_bus.send(ReadMultiTurnAngleMessage::new(node_id)).await?;
    sleep(Duration::from_secs_f32(0.1)).await;

    can_bus.send(PositionControlCommand::new(node_id, 0.0, 500)).await?;
    sleep(Duration::from_secs_f32(3.5)).await;

    can_bus.send(SpeedControlCommand::new(node_id, 100.0)).await?;
    sleep(Duration::from_secs_f32(3.5)).await;

    can_bus.send(SpeedControlCommand::new(node_id, 0.0)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    can_bus.send(SpeedControlCommand::new(node_id, -100.0)).await?;
    sleep(Duration::from_secs_f32(3.5)).await;

    can_bus.send(SpeedControlCommand::new(node_id, 0.0)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    can_bus.send(MotorShutdownCommand::new(node_id)).await?;
    sleep(Duration::from_secs_f32(0.5)).await;

    listen_task.abort();
    can_bus.shutdown().await;

    Ok(())
}
