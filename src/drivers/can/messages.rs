use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct RawCanMessage {
    pub arbitration_id: u32,
    pub data: Vec<u8>,
    pub is_extended_id: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdriveArbitrationId {
    pub node_id: u32,
    pub cmd_id: u32,
}

impl OdriveArbitrationId {
    pub fn from_can_message(msg: &RawCanMessage) -> Self {
        Self {
            node_id: msg.arbitration_id >> 5,
            cmd_id: msg.arbitration_id & 0b11111,
        }
    }

    pub fn value(&self) -> u32 {
        (self.node_id << 5) | self.cmd_id
    }
}

impl Hash for OdriveArbitrationId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
        self.cmd_id.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyActuatorArbitrationId {
    pub node_id: u32,
    pub cmd_id: u32,
    pub custom_value: Option<u32>,
}

impl MyActuatorArbitrationId {
    pub fn from_can_message(msg: &RawCanMessage) -> Result<Self, &'static str> {
        if (0x140..0x160).contains(&msg.arbitration_id) {
            Ok(Self {
                node_id: msg.arbitration_id - 0x140,
                cmd_id: if !msg.data.is_empty() { msg.data[0] as u32 } else { return Err("No data for cmd_id"); },
                custom_value: None,
            })
        } else if (0x240..0x260).contains(&msg.arbitration_id) {
            Ok(Self {
                node_id: msg.arbitration_id - 0x240,
                cmd_id: if !msg.data.is_empty() { msg.data[0] as u32 } else { return Err("No data for cmd_id"); },
                custom_value: None,
            })
        } else {
            Err("Invalid MyActuator arbitration ID")
        }
    }

    pub fn value(&self) -> u32 {
        self.custom_value.unwrap_or(0x140 + self.node_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct X424ArbitrationId {
    pub node_id: u32,
    pub cmd_id: u32,
}

impl X424ArbitrationId {
    pub fn from_can_message(msg: &RawCanMessage) -> Self {
        Self {
            node_id: msg.arbitration_id,
            cmd_id: if !msg.data.is_empty() { msg.data[0] as u32 } else { 0 },
        }
    }

    pub fn value(&self) -> u32 {
        self.node_id
    }
}

pub enum ArbitrationId {
    Odrive(OdriveArbitrationId),
    MyActuator(MyActuatorArbitrationId),
    X424(X424ArbitrationId),
}

pub trait CanMessageTrait {
    fn cmd_id() -> u32 where Self: Sized;

    fn node_id(&self) -> u32;

    fn matches(msg: &RawCanMessage) -> bool where Self: Sized;

    fn from_can_message(msg: RawCanMessage) -> Self where Self: Sized;

    fn as_can_message(&self) -> RawCanMessage;

    fn gen_arbitration_id(&self) -> ArbitrationId;

    fn gen_can_msg_data(&self) -> Vec<u8>;

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage);
}
