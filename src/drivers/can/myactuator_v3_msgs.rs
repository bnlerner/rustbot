use crate::drivers::can::enums::{MyActuatorFunctionControlIndex, MyActuatorV3OperatingMode};
use crate::drivers::can::messages::{ArbitrationId, CanMessageTrait, MyActuatorArbitrationId, RawCanMessage};
use chrono::NaiveDate;

// Helper function for clipping
fn clip(val: i32, min_val: i32, max_val: i32) -> i32 {
    val.max(min_val).min(max_val)
}

#[derive(Debug, Clone)]
pub struct MyActuatorCanMessage {
    pub node_id: u32,
    pub arbitration_id: MyActuatorArbitrationId,
}

impl MyActuatorCanMessage {
    pub fn new(node_id: u32, cmd_id: u32) -> Self {
        let arbitration_id = MyActuatorArbitrationId {
            node_id,
            cmd_id,
            custom_value: None,
        };
        Self { node_id, arbitration_id }
    }
}

impl CanMessageTrait for MyActuatorCanMessage {
    fn cmd_id() -> u32 where Self: Sized { 0 }

    fn node_id(&self) -> u32 { self.node_id }

    fn matches(msg: &RawCanMessage) -> bool where Self: Sized {
        (0x140..0x160).contains(&msg.arbitration_id) || (0x240..0x260).contains(&msg.arbitration_id)
    }

    fn from_can_message(msg: RawCanMessage) -> Self where Self: Sized {
        let arb_id = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb_id.node_id, arb_id.cmd_id);
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
        ArbitrationId::MyActuator(self.arbitration_id.clone())
    }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        vec![0; 8]
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct MyactuatorReadMotorStatus1Message {
    base: MyActuatorCanMessage,
    pub temperature: i8,
    pub brake_released: bool,
    pub voltage: f32,
    pub error_state: u16,
}

impl MyactuatorReadMotorStatus1Message {
    pub fn new(node_id: u32) -> Self {
        Self {
            base: MyActuatorCanMessage::new(node_id, Self::cmd_id()),
            temperature: 0,
            brake_released: false,
            voltage: 0.0,
            error_state: 0,
        }
    }
}

impl CanMessageTrait for MyactuatorReadMotorStatus1Message {
    fn cmd_id() -> u32 { 0x9A }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let mut s = Self::new(0); // temp node_id
        s.parse_can_msg_data(&msg);
        s.base = MyActuatorCanMessage::new(s.node_id(), Self::cmd_id());
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.temperature = msg.data[1] as i8;
        self.brake_released = msg.data[3] != 0;
        let voltage_raw = ((msg.data[5] as u16) << 8) | msg.data[4] as u16;
        self.voltage = voltage_raw as f32 * 0.1;
        self.error_state = ((msg.data[7] as u16) << 8) | msg.data[6] as u16;
        // Set node_id from arb
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct ReadMotorStatus2Message {
    base: MyActuatorCanMessage,
    pub temperature: i8,
    pub torque_current: f32,
    pub speed: i16,
    pub angle: i16,
}

impl ReadMotorStatus2Message {
    pub fn new(node_id: u32) -> Self {
        Self {
            base: MyActuatorCanMessage::new(node_id, Self::cmd_id()),
            temperature: 0,
            torque_current: 0.0,
            speed: 0,
            angle: 0,
        }
    }
}

impl CanMessageTrait for ReadMotorStatus2Message {
    fn cmd_id() -> u32 { 0x9C }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let mut s = Self::new(0);
        s.parse_can_msg_data(&msg);
        s.base = MyActuatorCanMessage::new(s.node_id(), Self::cmd_id());
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.temperature = msg.data[1] as i8;
        let mut current_raw = ((msg.data[3] as i32) << 8) | msg.data[2] as i32;
        if current_raw > 32767 { current_raw -= 65536; }
        self.torque_current = current_raw as f32 * 0.01;
        let mut speed_raw = ((msg.data[5] as i32) << 8) | msg.data[4] as i32;
        if speed_raw > 32767 { speed_raw -= 65536; }
        self.speed = speed_raw as i16;
        let mut angle_raw = ((msg.data[7] as i32) << 8) | msg.data[6] as i32;
        if angle_raw > 32767 { angle_raw -= 65536; }
        self.angle = angle_raw as i16;
        // Set node_id
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct WriteMotorZeroPositionMessage {
    base: MyActuatorCanMessage,
}

impl WriteMotorZeroPositionMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for WriteMotorZeroPositionMessage {
    fn cmd_id() -> u32 { 0x64 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct TorqueControlCommand {
    base: MyActuatorCanMessage,
    pub torque_current: f32,
}

impl TorqueControlCommand {
    pub fn new(node_id: u32, torque_current: f32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), torque_current }
    }
}

impl CanMessageTrait for TorqueControlCommand {
    fn cmd_id() -> u32 { 0xA1 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let torque_raw = (self.torque_current * 100.0) as i16;
        vec![Self::cmd_id() as u8, 0, 0, 0, (torque_raw & 0xFF) as u8, ((torque_raw >> 8) & 0xFF) as u8, 0, 0]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 6 { return; }
        let torque_raw = ((msg.data[5] as i16) << 8) | msg.data[4] as i16;
        self.torque_current = torque_raw as f32 * 0.01;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct FunctionControlCommand {
    base: MyActuatorCanMessage,
    pub function: MyActuatorFunctionControlIndex,
    pub function_value: i32,
}

impl FunctionControlCommand {
    pub fn new(node_id: u32, function: MyActuatorFunctionControlIndex, function_value: i32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), function, function_value }
    }
}

impl CanMessageTrait for FunctionControlCommand {
    fn cmd_id() -> u32 { 0x20 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, MyActuatorFunctionControlIndex::ClearMultiTurnValue, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        vec![
            Self::cmd_id() as u8,
            self.function.value() as u8,
            0,
            0,
            (self.function_value & 0xFF) as u8,
            ((self.function_value >> 8) & 0xFF) as u8,
            ((self.function_value >> 16) & 0xFF) as u8,
            ((self.function_value >> 24) & 0xFF) as u8,
        ]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.function = MyActuatorFunctionControlIndex::from_value(msg.data[1]).unwrap_or(MyActuatorFunctionControlIndex::ClearMultiTurnValue);
        self.function_value = ((msg.data[7] as i32) << 24) | ((msg.data[6] as i32) << 16) | ((msg.data[5] as i32) << 8) | (msg.data[4] as i32);
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct SpeedControlCommand {
    base: MyActuatorCanMessage,
    pub speed: f32,
}

impl SpeedControlCommand {
    pub fn new(node_id: u32, speed: f32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), speed }
    }
}

impl CanMessageTrait for SpeedControlCommand {
    fn cmd_id() -> u32 { 0xA2 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let speed_raw = (self.speed * 100.0) as i32;
        vec![
            Self::cmd_id() as u8,
            0, 0, 0,
            (speed_raw & 0xFF) as u8,
            ((speed_raw >> 8) & 0xFF) as u8,
            ((speed_raw >> 16) & 0xFF) as u8,
            ((speed_raw >> 24) & 0xFF) as u8,
        ]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        let speed_raw = ((msg.data[7] as i32) << 24) | ((msg.data[6] as i32) << 16) | ((msg.data[5] as i32) << 8) | msg.data[4] as i32;
        self.speed = speed_raw as f32 / 100.0;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct PositionControlCommand {
    base: MyActuatorCanMessage,
    pub position: f32,
    pub max_speed: u16,
}

impl PositionControlCommand {
    pub fn new(node_id: u32, position: f32, max_speed: u16) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), position, max_speed }
    }
}

impl CanMessageTrait for PositionControlCommand {
    fn cmd_id() -> u32 { 0xA4 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, 0.0, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let position_raw = (self.position * 100.0) as i32;
        vec![
            Self::cmd_id() as u8,
            0,
            (self.max_speed & 0xFF) as u8,
            ((self.max_speed >> 8) & 0xFF) as u8,
            (position_raw & 0xFF) as u8,
            ((position_raw >> 8) & 0xFF) as u8,
            ((position_raw >> 16) & 0xFF) as u8,
            ((position_raw >> 24) & 0xFF) as u8,
        ]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.max_speed = ((msg.data[3] as u16) << 8) | msg.data[2] as u16;
        let position_raw = ((msg.data[7] as i32) << 24) | ((msg.data[6] as i32) << 16) | ((msg.data[5] as i32) << 8) | msg.data[4] as i32;
        self.position = position_raw as f32 / 100.0;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalPositionControlCommand {
    base: MyActuatorCanMessage,
    pub max_speed: u16,
    pub position_increment: f32,
}

impl IncrementalPositionControlCommand {
    pub fn new(node_id: u32, max_speed: u16, position_increment: f32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), max_speed, position_increment }
    }
}

impl CanMessageTrait for IncrementalPositionControlCommand {
    fn cmd_id() -> u32 { 0xA8 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, 0, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let position_raw = (self.position_increment * 100.0) as i32;
        vec![
            Self::cmd_id() as u8,
            0,
            (self.max_speed & 0xFF) as u8,
            ((self.max_speed >> 8) & 0xFF) as u8,
            (position_raw & 0xFF) as u8,
            ((position_raw >> 8) & 0xFF) as u8,
            ((position_raw >> 16) & 0xFF) as u8,
            ((position_raw >> 24) & 0xFF) as u8,
        ]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.max_speed = ((msg.data[3] as u16) << 8) | msg.data[2] as u16;
        let position_raw = ((msg.data[7] as i32) << 24) | ((msg.data[6] as i32) << 16) | ((msg.data[5] as i32) << 8) | msg.data[4] as i32;
        self.position_increment = position_raw as f32 / 100.0;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct MotorShutdownCommand {
    base: MyActuatorCanMessage,
}

impl MotorShutdownCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for MotorShutdownCommand {
    fn cmd_id() -> u32 { 0x80 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct MotorStopCommand {
    base: MyActuatorCanMessage,
}

impl MotorStopCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for MotorStopCommand {
    fn cmd_id() -> u32 { 0x81 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct ReadMultiTurnAngleMessage {
    base: MyActuatorCanMessage,
    pub angle: f32,
}

impl ReadMultiTurnAngleMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), angle: 0.0 }
    }
}

impl CanMessageTrait for ReadMultiTurnAngleMessage {
    fn cmd_id() -> u32 { 0x92 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        let mut angle_raw = ((msg.data[7] as i64) << 24) | ((msg.data[6] as i64) << 16) | ((msg.data[5] as i64) << 8) | msg.data[4] as i64;
        if angle_raw > 0x7FFFFFFF { angle_raw -= 0x100000000i64; }
        self.angle = angle_raw as f32 * 0.01;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct SystemBrakeReleaseCommand {
    base: MyActuatorCanMessage,
}

impl SystemBrakeReleaseCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for SystemBrakeReleaseCommand {
    fn cmd_id() -> u32 { 0x77 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct SystemBrakeLockCommand {
    base: MyActuatorCanMessage,
}

impl SystemBrakeLockCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for SystemBrakeLockCommand {
    fn cmd_id() -> u32 { 0x78 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct SystemOperatingModeAcquisitionCommand {
    base: MyActuatorCanMessage,
    pub operating_mode: MyActuatorV3OperatingMode,
}

impl SystemOperatingModeAcquisitionCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), operating_mode: MyActuatorV3OperatingMode::PositionLoopControl }
    }
}

impl CanMessageTrait for SystemOperatingModeAcquisitionCommand {
    fn cmd_id() -> u32 { 0x70 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.operating_mode = MyActuatorV3OperatingMode::from_value(msg.data[7]).unwrap_or(MyActuatorV3OperatingMode::PositionLoopControl);
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone)]
pub struct SystemResetCommand {
    base: MyActuatorCanMessage,
}

impl SystemResetCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for SystemResetCommand {
    fn cmd_id() -> u32 { 0x76 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        Self { base: MyActuatorCanMessage::new(arb.node_id, Self::cmd_id()) }
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct VersionAcquisitionCommand {
    base: MyActuatorCanMessage,
    pub version_date: u32,
}

impl VersionAcquisitionCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), version_date: 0 }
    }

    pub fn version_datetime(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.version_date.to_string(), "%Y%m%d").unwrap()
    }
}

impl CanMessageTrait for VersionAcquisitionCommand {
    fn cmd_id() -> u32 { 0xB2 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![Self::cmd_id() as u8, 0, 0, 0, 0, 0, 0, 0] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.version_date = ((msg.data[7] as u32) << 24) | ((msg.data[6] as u32) << 16) | ((msg.data[5] as u32) << 8) | msg.data[4] as u32;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadWriteFlag {
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct CANIDCommand {
    base: MyActuatorCanMessage,
    pub read_write_flag: ReadWriteFlag,
    pub can_id: u32,
}

impl CANIDCommand {
    pub fn new(node_id: u32, read_write_flag: ReadWriteFlag, can_id: u32) -> Self {
        Self { base: MyActuatorCanMessage::new(node_id, Self::cmd_id()), read_write_flag, can_id }
    }
}

impl CanMessageTrait for CANIDCommand {
    fn cmd_id() -> u32 { 0x79 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        MyActuatorCanMessage::matches(msg) && !msg.data.is_empty() && msg.data[0] == Self::cmd_id() as u8
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = MyActuatorArbitrationId::from_can_message(&msg).unwrap();
        let mut s = Self::new(arb.node_id, ReadWriteFlag::Write, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId {
        let mut arb = self.base.arbitration_id.clone();
        arb.custom_value = Some(0x300);
        ArbitrationId::MyActuator(arb)
    }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let flag_byte = if self.read_write_flag == ReadWriteFlag::Read { 0x01 } else { 0x00 };
        let clipped_id = clip(self.can_id as i32, 1, 32) as u8;
        vec![Self::cmd_id() as u8, 0, flag_byte, 0, 0, 0, 0, clipped_id]
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() < 8 { return; }
        self.read_write_flag = if msg.data[2] != 0 { ReadWriteFlag::Read } else { ReadWriteFlag::Write };
        self.can_id = msg.data[7] as u32;
        let arb = MyActuatorArbitrationId::from_can_message(msg).unwrap();
        self.base.node_id = arb.node_id;
    }
}
