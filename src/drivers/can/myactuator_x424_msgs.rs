use byteorder::{BigEndian, WriteBytesExt};
use std::io::Cursor;
use std::convert::TryInto;

use crate::drivers::can::messages::{ArbitrationId, CanMessageTrait, RawCanMessage, X424ArbitrationId};
use crate::drivers::can::enums::X424MotorError;

#[derive(Debug, Clone)]
pub struct X424CanMessage {
    pub node_id: u32,
    pub arbitration_id: X424ArbitrationId,
}

impl X424CanMessage {
    pub fn new(node_id: u32, cmd_id: u32) -> Self {
        let arbitration_id = X424ArbitrationId { node_id, cmd_id };
        Self { node_id, arbitration_id }
    }
}

impl CanMessageTrait for X424CanMessage {
    fn cmd_id() -> u32 { 0 }

    fn node_id(&self) -> u32 { self.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        msg.data.get(0).map_or(false, |&d| d == Self::cmd_id() as u8)
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, arb.cmd_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage {
        RawCanMessage {
            arbitration_id: self.arbitration_id.value(),
            data: self.gen_can_msg_data(),
            is_extended_id: false,
        }
    }

    fn gen_arbitration_id(&self) -> ArbitrationId {
        ArbitrationId::X424(self.arbitration_id.clone())
    }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct X424CanMessageSetAndQuery {
    base: X424CanMessage,
}

impl X424CanMessageSetAndQuery {
    pub fn new(node_id: u32, cmd_id: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, cmd_id) }
    }
}

impl CanMessageTrait for X424CanMessageSetAndQuery {
    fn cmd_id() -> u32 { 0 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        msg.arbitration_id == 0x7FF && msg.data.get(3).map_or(false, |&d| d == Self::cmd_id() as u8)
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, arb.cmd_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage {
        RawCanMessage {
            arbitration_id: 0x7FF,
            data: self.gen_can_msg_data(),
            is_extended_id: false,
        }
    }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct QueryCommunicationModeMessage {
    base: X424CanMessageSetAndQuery,
    pub mode: String,
}

impl QueryCommunicationModeMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()), mode: String::new() }
    }
}

impl CanMessageTrait for QueryCommunicationModeMessage {
    fn cmd_id() -> u32 { 0x81 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        msg.arbitration_id == 0x7FF && msg.data.get(2).map_or(false, |&d| d == 0x01)
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let high_id = ((self.base.base.node_id >> 8) & 0xFF) as u8;
        let low_id = (self.base.base.node_id & 0xFF) as u8;
        vec![high_id, low_id, 0x00, Self::cmd_id() as u8]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.mode = if msg.data[3] == 0x01 { "auto".to_string() } else { "qa".to_string() };
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryCANCommunicationIDMessage {
    base: X424CanMessageSetAndQuery,
}

impl QueryCANCommunicationIDMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for QueryCANCommunicationIDMessage {
    fn cmd_id() -> u32 { 0x82 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        msg.arbitration_id == 0x7FF && msg.data.get(2).map_or(false, |&d| d == 0x01)
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        vec![0xFF, 0xFF, 0x00, Self::cmd_id() as u8]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 5 && msg.data[0] == 0xFF && msg.data[2] == 0x01 {
            let data = &msg.data[3..5];
            self.base.base.node_id = u16::from_be_bytes(data.try_into().unwrap()) as u32;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetCommunicationModeMessage {
    base: X424CanMessageSetAndQuery,
    pub mode: String,
}

impl SetCommunicationModeMessage {
    pub fn new(node_id: u32, mode: String) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()), mode }
    }
}

impl CanMessageTrait for SetCommunicationModeMessage {
    fn cmd_id() -> u32 { 0x00 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessageSetAndQuery::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, String::new());
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let high_id = ((self.base.base.node_id >> 8) & 0xFF) as u8;
        let low_id = (self.base.base.node_id & 0xFF) as u8;
        let mode_val = if self.mode == "auto" { 0x01 } else { 0x02 };
        vec![high_id, low_id, Self::cmd_id() as u8, mode_val]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        self.base.base.node_id = msg.arbitration_id;
        if msg.data.len() >= 4 {
            self.mode = if msg.data[3] == 0x01 { "auto".to_string() } else { "qa".to_string() };
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetZeroPositionMessage {
    base: X424CanMessageSetAndQuery,
}

impl SetZeroPositionMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for SetZeroPositionMessage {
    fn cmd_id() -> u32 { 0x03 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessageSetAndQuery::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let high_id = ((self.base.base.node_id >> 8) & 0xFF) as u8;
        let low_id = (self.base.base.node_id & 0xFF) as u8;
        vec![high_id, low_id, 0x00, Self::cmd_id() as u8]
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct SetMotorIDMessage {
    base: X424CanMessageSetAndQuery,
    pub cur_node_id: u32,
    pub new_node_id: u32,
}

impl SetMotorIDMessage {
    pub fn new(node_id: u32, cur_node_id: u32, new_node_id: u32) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()), cur_node_id, new_node_id }
    }
}

impl CanMessageTrait for SetMotorIDMessage {
    fn cmd_id() -> u32 { 0x04 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessageSetAndQuery::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id, 0, 0)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let cur_high = ((self.cur_node_id >> 8) & 0xFF) as u8;
        let cur_low = (self.cur_node_id & 0xFF) as u8;
        let new_high = ((self.new_node_id >> 8) & 0xFF) as u8;
        let new_low = (self.new_node_id & 0xFF) as u8;
        vec![cur_high, cur_low, 0x00, Self::cmd_id() as u8, new_high, new_low]
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct ResetMotorIDMessage {
    base: X424CanMessageSetAndQuery,
}

impl ResetMotorIDMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessageSetAndQuery::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for ResetMotorIDMessage {
    fn cmd_id() -> u32 { 0x05 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessageSetAndQuery::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        vec![0x7F, 0x7F, 0x00, Self::cmd_id() as u8, 0x7F, 0x7F]
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct X424ServoPositionControlMessage {
    base: X424CanMessage,
    pub position: f32,
    pub speed: f32,
    pub current_limit: f32,
    pub message_type: u32,
}

impl X424ServoPositionControlMessage {
    pub fn new(node_id: u32, position: f32, speed: f32, current_limit: f32, message_type: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), position, speed, current_limit, message_type }
    }
}

impl CanMessageTrait for X424ServoPositionControlMessage {
    fn cmd_id() -> u32 { 0x01 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0, 0.0, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut result: u64 = 0;
        let speed_value = (self.speed.min(32767.0) as u32) & 0x7FFF;
        let current_value = (self.current_limit.min(409.5) * 10.0) as u32 & 0xFFF;
        let position_bytes = self.position.to_le_bytes();
        let position_int = u32::from_le_bytes(position_bytes);
        result |= ((Self::cmd_id() & 0x07) as u64) << 61;
        result |= ((position_int & 0xFFFFFFFF) as u64) << 29;
        result |= ((speed_value) as u64) << 14;
        result |= ((current_value) as u64) << 2;
        result |= (self.message_type & 0x03) as u64;
        result.to_be_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct X424ServoSpeedControlMessage {
    base: X424CanMessage,
    pub speed: f32,
    pub current_limit: f32,
    pub message_type: u32,
}

impl X424ServoSpeedControlMessage {
    pub fn new(node_id: u32, speed: f32, current_limit: f32, message_type: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), speed, current_limit, message_type }
    }
}

impl CanMessageTrait for X424ServoSpeedControlMessage {
    fn cmd_id() -> u32 { 0x02 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut result: u64 = 0;
        let current_value = (self.current_limit.min(6553.5) * 10.0) as u32 & 0xFFFF;
        let speed_bytes = self.speed.to_le_bytes();
        let speed_int = u32::from_le_bytes(speed_bytes);
        result |= ((Self::cmd_id() & 0x07) as u64) << 53;
        result |= (0u64 & 0x07) << 50;
        result |= ((self.message_type & 0x03) as u64) << 48;
        result |= ((speed_int & 0xFFFFFFFF) as u64) << 16;
        result |= current_value as u64;
        let mut bytes = vec![0u8; 7];
        let mut cursor = Cursor::new(&mut bytes);
        cursor.write_u64::<BigEndian>(result).unwrap();
        bytes
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct X424CurrentControlMessage {
    base: X424CanMessage,
    pub current: f32,
    pub control_type: u32,
    pub message_type: u32,
}

impl X424CurrentControlMessage {
    pub fn new(node_id: u32, current: f32, control_type: u32, message_type: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), current, control_type, message_type }
    }
}

impl CanMessageTrait for X424CurrentControlMessage {
    fn cmd_id() -> u32 { 0x03 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { X424CanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut result: u32 = 0;
        let mut current_int = (self.current * 100.0) as i32;
        if current_int < 0 {
            current_int = (current_int.abs() ^ 0xFFFF) + 1;
        }
        current_int &= 0xFFFF;
        result |= ((Self::cmd_id() & 0x07) as u32) << 21;
        result |= ((self.control_type & 0x07) as u32) << 18;
        result |= ((self.message_type & 0x03) as u32) << 16;
        result |= (current_int as u32) & 0xFFFF;
        result.to_be_bytes()[1..4].to_vec()
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct QAReturnMessage {
    base: X424CanMessage,
    pub motor_error: X424MotorError,
}

impl QAReturnMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), motor_error: X424MotorError::NoError }
    }
}

impl CanMessageTrait for QAReturnMessage {
    fn cmd_id() -> u32 { 0 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if !msg.data.is_empty() {
            self.motor_error = X424MotorError::from_value(msg.data[0] & 0x1F);
        }
    }
}

#[derive(Debug, Clone)]
pub struct QAReturnMessageType1 {
    base: QAReturnMessage,
    pub position: f32,
    pub speed: f32,
    pub current: f32,
    pub motor_temp: f32,
    pub mos_temp: f32,
}

impl QAReturnMessageType1 {
    pub fn new(node_id: u32) -> Self {
        Self { base: QAReturnMessage::new(node_id), position: 0.0, speed: 0.0, current: 0.0, motor_temp: 0.0, mos_temp: 0.0 }
    }
}

impl CanMessageTrait for QAReturnMessageType1 {
    fn cmd_id() -> u32 { 0x01 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() == 8 {
            let data_int = u64::from_be_bytes((&msg.data[..]).try_into().unwrap());
            let position_raw = ((data_int >> 40) & 0xFFFF) as u32;
            self.position = (position_raw as f32 / 65536.0 * 25.0) - 12.5;
            let speed_raw = ((data_int >> 28) & 0xFFF) as u32;
            self.speed = (speed_raw as f32 / 4095.0 * 36.0) - 18.0;
            let current_raw = ((data_int >> 16) & 0xFFF) as u32;
            self.current = (current_raw as f32 / 4095.0 * 60.0) - 30.0;
            let temp_raw = ((data_int >> 8) & 0xFF) as u32;
            self.motor_temp = (temp_raw as f32 - 50.0) / 2.0;
            let mos_temp_raw = (data_int & 0xFF) as u32;
            self.mos_temp = (mos_temp_raw as f32 - 50.0) / 2.0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct QAReturnMessageType2 {
    base: QAReturnMessage,
    pub position: f32,
    pub current: f32,
    pub motor_temp: f32,
}

impl QAReturnMessageType2 {
    pub fn new(node_id: u32) -> Self {
        Self { base: QAReturnMessage::new(node_id), position: 0.0, current: 0.0, motor_temp: 0.0 }
    }
}

impl CanMessageTrait for QAReturnMessageType2 {
    fn cmd_id() -> u32 { 0x02 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.position = f32::from_le_bytes([msg.data[1], msg.data[2], msg.data[3], msg.data[4]]);
            let mut current_raw = i16::from_be_bytes([msg.data[5], msg.data[6]]);
            if current_raw < 0 {
                current_raw = -current_raw;
            }
            self.current = current_raw as f32 / 100.0;
            let temp_raw = msg.data[7];
            self.motor_temp = (temp_raw as f32 - 50.0) / 2.0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct QAReturnMessageType3 {
    base: QAReturnMessage,
    pub speed: f32,
    pub current: f32,
    pub motor_temp: f32,
}

impl QAReturnMessageType3 {
    pub fn new(node_id: u32) -> Self {
        Self { base: QAReturnMessage::new(node_id), speed: 0.0, current: 0.0, motor_temp: 0.0 }
    }
}

impl CanMessageTrait for QAReturnMessageType3 {
    fn cmd_id() -> u32 { 0x03 }

    fn node_id(&self) -> u32 { self.base.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.speed = f32::from_le_bytes([msg.data[1], msg.data[2], msg.data[3], msg.data[4]]);
            let mut current_raw = i16::from_be_bytes([msg.data[5], msg.data[6]]);
            if current_raw < 0 {
                current_raw = -current_raw;
            }
            self.current = current_raw as f32 / 100.0;
            let temp_raw = msg.data[7];
            self.motor_temp = (temp_raw as f32 - 50.0) / 2.0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct QAReturnMessageType4 {
    base: X424CanMessage,
    pub config_code: u8,
    pub config_status: bool,
}

impl QAReturnMessageType4 {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), config_code: 0, config_status: false }
    }
}

impl CanMessageTrait for QAReturnMessageType4 {
    fn cmd_id() -> u32 { 0x04 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 3 {
            self.config_code = msg.data[1];
            self.config_status = msg.data[2] == 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct QAReturnMessageType5 {
    base: X424CanMessage,
    pub query_code: u8,
    pub position: f32,
    pub speed: f32,
    pub current: f32,
    pub power: f32,
    pub uint16_value: u16,
}

impl QAReturnMessageType5 {
    pub fn new(node_id: u32) -> Self {
        Self { base: X424CanMessage::new(node_id, Self::cmd_id()), query_code: 0, position: 0.0, speed: 0.0, current: 0.0, power: 0.0, uint16_value: 0 }
    }
}

impl CanMessageTrait for QAReturnMessageType5 {
    fn cmd_id() -> u32 { 0x05 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        if msg.data.is_empty() { return false; }
        ((msg.data[0] >> 5) & 0x07) as u32 == Self::cmd_id()
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = X424ArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 3 {
            self.query_code = msg.data[1];
            if self.query_code >= 1 && self.query_code <= 4 && msg.data.len() >= 6 {
                let value = f32::from_le_bytes([msg.data[2], msg.data[3], msg.data[4], msg.data[5]]);
                match self.query_code {
                    1 => self.position = value,
                    2 => self.speed = value,
                    3 => self.current = value,
                    4 => self.power = value,
                    _ => {},
                }
            } else if self.query_code >= 5 && self.query_code <= 9 && msg.data.len() >= 4 {
                self.uint16_value = u16::from_be_bytes([msg.data[2], msg.data[3]]);
            }
        }
    }
}
