use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::convert::TryInto;

use crate::drivers::can::messages::{ArbitrationId, CanMessageTrait, OdriveArbitrationId, RawCanMessage};
use crate::drivers::can::enums::{AxisState, ControlMode, InputMode, ODriveError, ProcedureResult, ValueTypes};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Uint64(u64),
    Int64(i64),
    Float(f32),
}

#[derive(Debug, Clone)]
pub struct OdriveCanMessage {
    pub node_id: u32,
    pub arbitration_id: OdriveArbitrationId,
}

impl OdriveCanMessage {
    pub fn new(node_id: u32, cmd_id: u32) -> Self {
        let arbitration_id = OdriveArbitrationId { node_id, cmd_id };
        Self { node_id, arbitration_id }
    }
}

impl CanMessageTrait for OdriveCanMessage {
    fn cmd_id() -> u32 { 0 }

    fn node_id(&self) -> u32 { self.node_id }

    fn matches(msg: &RawCanMessage) -> bool {
        let arb = OdriveArbitrationId::from_can_message(msg);
        Self::cmd_id() == arb.cmd_id
    }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
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
        ArbitrationId::Odrive(self.arbitration_id.clone())
    }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

// Cyclic Messages

#[derive(Debug, Clone)]
pub struct BusVoltageCurrentMessage {
    base: OdriveCanMessage,
    pub voltage: f32,
    pub current: f32,
}

impl BusVoltageCurrentMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), voltage: 0.0, current: 0.0 }
    }
}

impl CanMessageTrait for BusVoltageCurrentMessage {
    fn cmd_id() -> u32 { 0x17 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.voltage = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.current = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

// Add the remaining cyclic messages

#[derive(Debug, Clone)]
pub struct EncoderEstimatesMessage {
    base: OdriveCanMessage,
    pub pos_estimate: f32,
    pub vel_estimate: f32,
}

impl EncoderEstimatesMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), pos_estimate: 0.0, vel_estimate: 0.0 }
    }
}

impl CanMessageTrait for EncoderEstimatesMessage {
    fn cmd_id() -> u32 { 0x09 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.pos_estimate = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.vel_estimate = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMessage {
    base: OdriveCanMessage,
    pub active_errors: Vec<ODriveError>,
    pub disarm_reason: Vec<ODriveError>,
}

impl ErrorMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), active_errors: vec![], disarm_reason: vec![] }
    }
}

impl CanMessageTrait for ErrorMessage {
    fn cmd_id() -> u32 { 0x03 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            let mut cursor = Cursor::new(&msg.data);
            let active_errors_int = cursor.read_u32::<LittleEndian>().unwrap();
            self.active_errors = ODriveError::from_bits(active_errors_int);
            let disarm_reason_int = cursor.read_u32::<LittleEndian>().unwrap();
            self.disarm_reason = ODriveError::from_bits(disarm_reason_int);
        }
    }
}

// HeartbeatMessage already implemented

#[derive(Debug, Clone)]
pub struct IqMessage {
    base: OdriveCanMessage,
    pub setpoint: f32,
    pub measured: f32,
}

impl IqMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), setpoint: 0.0, measured: 0.0 }
    }
}

impl CanMessageTrait for IqMessage {
    fn cmd_id() -> u32 { 0x14 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.setpoint = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.measured = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct PowersMessage {
    base: OdriveCanMessage,
    pub electrical_power: f32,
    pub mechanical_power: f32,
}

impl PowersMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), electrical_power: 0.0, mechanical_power: 0.0 }
    }
}

impl CanMessageTrait for PowersMessage {
    fn cmd_id() -> u32 { 0x1D }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.electrical_power = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.mechanical_power = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemperatureMessage {
    base: OdriveCanMessage,
    pub fet_temperature: f32,
    pub motor_temperature: f32,
}

impl TemperatureMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), fet_temperature: 0.0, motor_temperature: 0.0 }
    }
}

impl CanMessageTrait for TemperatureMessage {
    fn cmd_id() -> u32 { 0x15 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.fet_temperature = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.motor_temperature = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct TorquesMessage {
    base: OdriveCanMessage,
    pub target: f32,
    pub estimate: f32,
}

impl TorquesMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), target: 0.0, estimate: 0.0 }
    }
}

impl CanMessageTrait for TorquesMessage {
    fn cmd_id() -> u32 { 0x1C }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.target = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.estimate = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersionMessage {
    base: OdriveCanMessage,
    pub hw_major: u8,
    pub hw_minor: u8,
    pub hw_variant: u8,
    pub fw_major: u8,
    pub fw_minor: u8,
    pub fw_revision: u8,
}

impl VersionMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), hw_major: 0, hw_minor: 0, hw_variant: 0, fw_major: 0, fw_minor: 0, fw_revision: 0 }
    }

    pub fn hw_version(&self) -> String {
        format!("{}.{}.{}", self.hw_major, self.hw_minor, self.hw_variant)
    }

    pub fn fw_version(&self) -> String {
        format!("{}.{}.{}", self.fw_major, self.fw_minor, self.fw_revision)
    }
}

impl CanMessageTrait for VersionMessage {
    fn cmd_id() -> u32 { 0x00 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            let _protocol = msg.data[0];
            self.hw_major = msg.data[1];
            self.hw_minor = msg.data[2];
            self.hw_variant = msg.data[3];
            self.fw_major = msg.data[4];
            self.fw_minor = msg.data[5];
            self.fw_revision = msg.data[6];
            let _unreleased = msg.data[7];
        }
    }
}

// HeartbeatMessage already implemented

#[derive(Debug, Clone)]
pub struct HeartbeatMessage {
    base: OdriveCanMessage,
    pub axis_error: u32,
    pub axis_state: AxisState,
    pub procedure_result: ProcedureResult,
    pub trajectory_done: bool,
}

impl HeartbeatMessage {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), axis_error: 0, axis_state: AxisState::Undefined, procedure_result: ProcedureResult::Success, trajectory_done: false }
    }
}

impl CanMessageTrait for HeartbeatMessage {
    fn cmd_id() -> u32 { 0x01 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 7 {
            let mut cursor = Cursor::new(&msg.data);
            self.axis_error = cursor.read_u32::<LittleEndian>().unwrap();
            self.axis_state = AxisState::from(cursor.read_u8().unwrap());
            self.procedure_result = ProcedureResult::from(cursor.read_u8().unwrap());
            self.trajectory_done = cursor.read_u8().unwrap() != 0;
        }
    }
}

// Command messages

#[derive(Debug, Clone)]
pub struct ClearErrorsCommand {
    base: OdriveCanMessage,
    pub identify: u8,
}

impl ClearErrorsCommand {
    pub fn new(node_id: u32, identify: u8) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), identify }
    }
}

impl CanMessageTrait for ClearErrorsCommand {
    fn cmd_id() -> u32 { 0x18 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id, 0)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        vec![self.identify]
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct ReadParameterCommand {
    base: OdriveCanMessage,
    pub endpoint_id: u16,
}

impl ReadParameterCommand {
    pub fn new(node_id: u32, endpoint_id: u16) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), endpoint_id }
    }
}

impl CanMessageTrait for ReadParameterCommand {
    fn cmd_id() -> u32 { 0x04 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = vec![0u8; 4];
        data[0] = 0; // opcode read
        data[1..3].copy_from_slice(&self.endpoint_id.to_le_bytes());
        data[3] = 0; // reserved
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            let _opcode = msg.data[0];
            self.endpoint_id = u16::from_le_bytes([msg.data[1], msg.data[2]]);
            let _reserved = msg.data[3];
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteParameterCommand {
    base: OdriveCanMessage,
    pub endpoint_id: u16,
    pub value_type: ValueTypes,
    pub value: Value,
}

impl WriteParameterCommand {
    pub fn new(node_id: u32, endpoint_id: u16, value_type: ValueTypes, value: Value) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), endpoint_id, value_type, value }
    }

    fn pack_value(&self) -> Vec<u8> {
        match &self.value {
            Value::Bool(v) => vec![if *v { 1 } else { 0 }],
            Value::Uint8(v) => vec![*v],
            Value::Int8(v) => vec![*v as u8],
            Value::Uint16(v) => v.to_le_bytes().to_vec(),
            Value::Int16(v) => v.to_le_bytes().to_vec(),
            Value::Uint32(v) => v.to_le_bytes().to_vec(),
            Value::Int32(v) => v.to_le_bytes().to_vec(),
            Value::Float(v) => v.to_le_bytes().to_vec(),
            Value::Uint64(v) => v.to_le_bytes().to_vec(),
            Value::Int64(v) => v.to_le_bytes().to_vec(),
        }
    }
}

impl CanMessageTrait for WriteParameterCommand {
    fn cmd_id() -> u32 { 0x04 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id, 0, ValueTypes::Uint32, Value::Uint32(0))
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = vec![1u8; 4]; // opcode write
        data[1..3].copy_from_slice(&self.endpoint_id.to_le_bytes());
        data[3] = 0; // reserved
        data.extend(self.pack_value());
        data
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

// ParameterResponse already partially implemented

#[derive(Debug, Clone)]
pub struct ParameterResponse {
    base: OdriveCanMessage,
    pub endpoint_id: u16,
    pub value_type: ValueTypes,
    pub value: Value,
}

impl ParameterResponse {
    pub fn new(node_id: u32, endpoint_id: u16, value_type: ValueTypes, value: Value) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), endpoint_id, value_type, value }
    }

    pub fn parse_value(data: &[u8], value_type: ValueTypes) -> Value {
        match value_type {
            ValueTypes::Bool => Value::Bool(data[0] != 0),
            ValueTypes::Uint8 => Value::Uint8(data[0]),
            ValueTypes::Int8 => Value::Int8(data[0] as i8),
            ValueTypes::Uint16 => Value::Uint16(u16::from_le_bytes(data[0..2].try_into().unwrap())),
            ValueTypes::Int16 => Value::Int16(i16::from_le_bytes(data[0..2].try_into().unwrap())),
            ValueTypes::Uint32 => Value::Uint32(u32::from_le_bytes(data[0..4].try_into().unwrap())),
            ValueTypes::Int32 => Value::Int32(i32::from_le_bytes(data[0..4].try_into().unwrap())),
            ValueTypes::Float => Value::Float(f32::from_le_bytes(data[0..4].try_into().unwrap())),
            ValueTypes::Uint64 => Value::Uint64(u64::from_le_bytes(data[0..8].try_into().unwrap())),
            ValueTypes::Int64 => Value::Int64(i64::from_le_bytes(data[0..8].try_into().unwrap())),
        }
    }
}

impl CanMessageTrait for ParameterResponse {
    fn cmd_id() -> u32 { 0x05 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0, ValueTypes::Uint32, Value::Uint32(0));
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 + self.value_type.byte_size() {
            let mut cursor = Cursor::new(&msg.data);
            let _reserved0 = cursor.read_u8().unwrap();
            self.endpoint_id = cursor.read_u16::<LittleEndian>().unwrap();
            let _reserved1 = cursor.read_u8().unwrap();
            let value_data = &msg.data[4..4 + self.value_type.byte_size()];
            self.value = Self::parse_value(value_data, self.value_type);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetAxisStateMessage {
    base: OdriveCanMessage,
    pub axis_state: AxisState,
}

impl SetAxisStateMessage {
    pub fn new(node_id: u32, axis_state: AxisState) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), axis_state }
    }
}

impl CanMessageTrait for SetAxisStateMessage {
    fn cmd_id() -> u32 { 0x07 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id, AxisState::Undefined)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        (self.axis_state as u32).to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

// Implement SetControllerMode, SetPositionMessage, SetTorqueMessage, SetVelocityMessage, EStop, Reboot similarly

#[derive(Debug, Clone)]
pub struct SetControllerMode {
    base: OdriveCanMessage,
    pub control_mode: ControlMode,
    pub input_mode: InputMode,
}

impl SetControllerMode {
    pub fn new(node_id: u32, control_mode: ControlMode, input_mode: InputMode) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), control_mode, input_mode }
    }
}

impl CanMessageTrait for SetControllerMode {
    fn cmd_id() -> u32 { 0x0B }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, ControlMode::VoltageControl, InputMode::Inactive);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = (self.control_mode as u32).to_le_bytes().to_vec();
        data.extend_from_slice(&(self.input_mode as u32).to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.control_mode = ControlMode::from(u32::from_le_bytes(msg.data[0..4].try_into().unwrap()));
            self.input_mode = InputMode::from(u32::from_le_bytes(msg.data[4..8].try_into().unwrap()));
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetPositionMessage {
    base: OdriveCanMessage,
    pub input_position: f32,
    pub velocity_ff: i16,
    pub torque_ff: i16,
}

impl SetPositionMessage {
    pub fn new(node_id: u32, input_position: f32, velocity_ff: i16, torque_ff: i16) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), input_position, velocity_ff, torque_ff }
    }
}

impl CanMessageTrait for SetPositionMessage {
    fn cmd_id() -> u32 { 0x0C }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = self.input_position.to_le_bytes().to_vec();
        data.extend_from_slice(&self.velocity_ff.to_le_bytes());
        data.extend_from_slice(&self.torque_ff.to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.input_position = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.velocity_ff = i16::from_le_bytes(msg.data[4..6].try_into().unwrap());
            self.torque_ff = i16::from_le_bytes(msg.data[6..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetTorqueMessage {
    base: OdriveCanMessage,
    pub input_torque: f32,
}

impl SetTorqueMessage {
    pub fn new(node_id: u32, input_torque: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), input_torque }
    }
}

impl CanMessageTrait for SetTorqueMessage {
    fn cmd_id() -> u32 { 0x0E }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        self.input_torque.to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.input_torque = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetVelocityMessage {
    base: OdriveCanMessage,
    pub velocity: f32,
    pub torque: f32,
}

impl SetVelocityMessage {
    pub fn new(node_id: u32, velocity: f32, torque: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), velocity, torque }
    }
}

impl CanMessageTrait for SetVelocityMessage {
    fn cmd_id() -> u32 { 0x0D }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = self.velocity.to_le_bytes().to_vec();
        data.extend_from_slice(&self.torque.to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.velocity = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.torque = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct EStop {
    base: OdriveCanMessage,
}

impl EStop {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for EStop {
    fn cmd_id() -> u32 { 0x02 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}

#[derive(Debug, Clone)]
pub struct Reboot {
    base: OdriveCanMessage,
    pub action: u32,
}

impl Reboot {
    pub fn new(node_id: u32, action: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), action }
    }
}

impl CanMessageTrait for Reboot {
    fn cmd_id() -> u32 { 0x16 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { self.action.to_le_bytes().to_vec() }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.action = u32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetLimitsCommand {
    base: OdriveCanMessage,
    pub velocity_limit: f32,
    pub current_limit: f32,
}

impl SetLimitsCommand {
    pub fn new(node_id: u32, velocity_limit: f32, current_limit: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), velocity_limit, current_limit }
    }
}

impl CanMessageTrait for SetLimitsCommand {
    fn cmd_id() -> u32 { 0x0F }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = self.velocity_limit.to_le_bytes().to_vec();
        data.extend_from_slice(&self.current_limit.to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.velocity_limit = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.current_limit = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetTrajVelLimitMessage {
    base: OdriveCanMessage,
    pub traj_vel_limit: f32,
}

impl SetTrajVelLimitMessage {
    pub fn new(node_id: u32, traj_vel_limit: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), traj_vel_limit }
    }
}

impl CanMessageTrait for SetTrajVelLimitMessage {
    fn cmd_id() -> u32 { 0x11 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        self.traj_vel_limit.to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.traj_vel_limit = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetTrajAccelLimitsMessage {
    base: OdriveCanMessage,
    pub traj_accel_limit: f32,
    pub traj_decel_limit: f32,
}

impl SetTrajAccelLimitsMessage {
    pub fn new(node_id: u32, traj_accel_limit: f32, traj_decel_limit: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), traj_accel_limit, traj_decel_limit }
    }
}

impl CanMessageTrait for SetTrajAccelLimitsMessage {
    fn cmd_id() -> u32 { 0x12 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = self.traj_accel_limit.to_le_bytes().to_vec();
        data.extend_from_slice(&self.traj_decel_limit.to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.traj_accel_limit = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.traj_decel_limit = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetTrajInertiaMessage {
    base: OdriveCanMessage,
    pub traj_inertia: f32,
}

impl SetTrajInertiaMessage {
    pub fn new(node_id: u32, traj_inertia: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), traj_inertia }
    }
}

impl CanMessageTrait for SetTrajInertiaMessage {
    fn cmd_id() -> u32 { 0x13 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        self.traj_inertia.to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.traj_inertia = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetAbsolutePositionMessage {
    base: OdriveCanMessage,
    pub position: f32,
}

impl SetAbsolutePositionMessage {
    pub fn new(node_id: u32, position: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), position }
    }
}

impl CanMessageTrait for SetAbsolutePositionMessage {
    fn cmd_id() -> u32 { 0x019 }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        self.position.to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.position = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetPosGainMessage {
    base: OdriveCanMessage,
    pub pos_gain: f32,
}

impl SetPosGainMessage {
    pub fn new(node_id: u32, pos_gain: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), pos_gain }
    }
}

impl CanMessageTrait for SetPosGainMessage {
    fn cmd_id() -> u32 { 0x01A }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        self.pos_gain.to_le_bytes().to_vec()
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 4 {
            self.pos_gain = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetVelGainsMessage {
    base: OdriveCanMessage,
    pub vel_gain: f32,
    pub vel_integrator_gain: f32,
}

impl SetVelGainsMessage {
    pub fn new(node_id: u32, vel_gain: f32, vel_integrator_gain: f32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()), vel_gain, vel_integrator_gain }
    }
}

impl CanMessageTrait for SetVelGainsMessage {
    fn cmd_id() -> u32 { 0x01B }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        let mut s = Self::new(arb.node_id, 0.0, 0.0);
        s.parse_can_msg_data(&msg);
        s
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> {
        let mut data = self.vel_gain.to_le_bytes().to_vec();
        data.extend_from_slice(&self.vel_integrator_gain.to_le_bytes());
        data
    }

    fn parse_can_msg_data(&mut self, msg: &RawCanMessage) {
        if msg.data.len() >= 8 {
            self.vel_gain = f32::from_le_bytes(msg.data[0..4].try_into().unwrap());
            self.vel_integrator_gain = f32::from_le_bytes(msg.data[4..8].try_into().unwrap());
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnterDfuModeCommand {
    base: OdriveCanMessage,
}

impl EnterDfuModeCommand {
    pub fn new(node_id: u32) -> Self {
        Self { base: OdriveCanMessage::new(node_id, Self::cmd_id()) }
    }
}

impl CanMessageTrait for EnterDfuModeCommand {
    fn cmd_id() -> u32 { 0x01F }

    fn node_id(&self) -> u32 { self.base.node_id }

    fn matches(msg: &RawCanMessage) -> bool { OdriveCanMessage::matches(msg) }

    fn from_can_message(msg: RawCanMessage) -> Self {
        let arb = OdriveArbitrationId::from_can_message(&msg);
        Self::new(arb.node_id)
    }

    fn as_can_message(&self) -> RawCanMessage { self.base.as_can_message() }

    fn gen_arbitration_id(&self) -> ArbitrationId { self.base.gen_arbitration_id() }

    fn gen_can_msg_data(&self) -> Vec<u8> { vec![] }

    fn parse_can_msg_data(&mut self, _msg: &RawCanMessage) {}
}
