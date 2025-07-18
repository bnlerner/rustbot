/// Common enums for the CAN bus protocols

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusType {
    SocketCan,
    Virtual,
}

impl BusType {
    pub fn value(&self) -> &'static str {
        match self {
            BusType::SocketCan => "socketcan",
            BusType::Virtual => "virtual",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanInterface {
    /// Specifies the CAN interfaces.
    Odrive,
    Myactuator,
    Virtual,
}

impl CanInterface {
    pub fn value(&self) -> &'static str {
        match self {
            CanInterface::Odrive => "can0",
            CanInterface::Myactuator => "can0",
            CanInterface::Virtual => "vcan",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X424MotorError {
    /// Specifies the error codes for the X4-24 motor.
    NoError,
    MotorOverheating,
    MotorOvercurrent,
    MotorVoltageTooLow,
    MotorEncoderError,
    MotorBrakeVoltageTooHigh,
    DrvDriveError,
}

impl X424MotorError {
    pub fn value(&self) -> u8 {
        match self {
            X424MotorError::NoError => 0,
            X424MotorError::MotorOverheating => 1,
            X424MotorError::MotorOvercurrent => 2,
            X424MotorError::MotorVoltageTooLow => 3,
            X424MotorError::MotorEncoderError => 4,
            X424MotorError::MotorBrakeVoltageTooHigh => 6,
            X424MotorError::DrvDriveError => 7,
        }
    }

    pub fn from_value(value: u8) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::MotorOverheating,
            2 => Self::MotorOvercurrent,
            3 => Self::MotorVoltageTooLow,
            4 => Self::MotorEncoderError,
            6 => Self::MotorBrakeVoltageTooHigh,
            7 => Self::DrvDriveError,
            _ => Self::NoError,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MyActuatorV3OperatingMode {
    /// Specifies the operating modes for the MyActuator controller V3.
    CurrentLoopControl,
    SpeedLoopControl,
    PositionLoopControl,
}

impl MyActuatorV3OperatingMode {
    pub fn value(&self) -> u8 {
        match self {
            MyActuatorV3OperatingMode::CurrentLoopControl => 0x01,
            MyActuatorV3OperatingMode::SpeedLoopControl => 0x02,
            MyActuatorV3OperatingMode::PositionLoopControl => 0x03,
        }
    }

    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::CurrentLoopControl),
            0x02 => Some(Self::SpeedLoopControl),
            0x03 => Some(Self::PositionLoopControl),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MyActuatorFunctionControlIndex {
    /// Function indices for the MyActuator V3 controller Function Control Command (0x20).
    ///
    /// These values correspond to section 2.34.4 of the MyActuator Controller V4.2 protocol.
    /// Each function controls a specific aspect of the motor controller's behavior.
    ///
    /// Clear motor multi-turn value, update zero point and save. It will take effect after restarting.
    ClearMultiTurnValue,
    /// Enable/disable CANID filter for communication efficiency.
    /// The value "1" means that the CANID filter is enabled, which can improve the
    /// efficiency of motor sending and receiving in CAN communication.
    /// The value "0" means the disabled CANID filter, which needs to be disabled when the
    /// multi-motor control command 0x280, 0x300 is required.
    /// This value will be saved in FLASH, and the written value will be recorded after
    /// power off.
    CanidFilterEnable,
    /// Enable/disable automatic error status reporting.
    /// The value "1" means that this function is enabled. After the motor appears in an
    /// error state, it actively sends the status command 0x9A to the bus with a sending
    /// cycle of 100ms. Stop sending after the error status disappears.
    /// The value "0" means the function is disabled.
    ErrorStatusTransmission,
    /// Enable/disable saving multi-turn value on power off.
    /// The value "1" means that this function is enabled, and the motor will save the
    /// current multi-turn value before powering off. The system defaults to single lap
    /// mode. It will take effect after restarting.
    MultiTurnSaveOnPowerOff,
    /// Set the CANID for the motor.
    /// The value means the CANID number that is going to be modified, which will be saved
    /// to ROM and take effect after a reboot.
    SetCanid,
    /// Set the maximum positive angle for position mode.
    /// The value represents the maximum positive angle value for the position operation
    /// mode, which is set and saved to ROM to take effect immediately.
    SetMaxPositiveAngle,
    /// Set the maximum negative angle for position mode.
    /// The value represents the maximum negative angle value for the position operation
    /// mode, which is set and saved to ROM to take effect immediately.
    SetMaxNegativeAngle,
}

impl MyActuatorFunctionControlIndex {
    pub fn value(&self) -> u8 {
        match self {
            MyActuatorFunctionControlIndex::ClearMultiTurnValue => 0x01,
            MyActuatorFunctionControlIndex::CanidFilterEnable => 0x02,
            MyActuatorFunctionControlIndex::ErrorStatusTransmission => 0x03,
            MyActuatorFunctionControlIndex::MultiTurnSaveOnPowerOff => 0x04,
            MyActuatorFunctionControlIndex::SetCanid => 0x05,
            MyActuatorFunctionControlIndex::SetMaxPositiveAngle => 0x06,
            MyActuatorFunctionControlIndex::SetMaxNegativeAngle => 0x07,
        }
    }

    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::ClearMultiTurnValue),
            0x02 => Some(Self::CanidFilterEnable),
            0x03 => Some(Self::ErrorStatusTransmission),
            0x04 => Some(Self::MultiTurnSaveOnPowerOff),
            0x05 => Some(Self::SetCanid),
            0x06 => Some(Self::SetMaxPositiveAngle),
            0x07 => Some(Self::SetMaxNegativeAngle),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisState {
    Undefined = 0,
    Idle = 1,
    StartupSequence = 2,
    FullCalibrationSequence = 3,
    MotorCalibration = 4,
    EncoderIndexSearch = 6,
    EncoderOffsetCalibration = 7,
    ClosedLoopControl = 8,
    LockinSpin = 9,
    EncoderDirFind = 10,
    Homing = 11,
    EncoderHallPolarityCalibration = 12,
    EncoderHallPhaseCalibration = 13,
    AnticoggingCalibration = 14,
}

impl From<u8> for AxisState {
    fn from(value: u8) -> Self {
        match value {
            1 => AxisState::Idle,
            2 => AxisState::StartupSequence,
            3 => AxisState::FullCalibrationSequence,
            4 => AxisState::MotorCalibration,
            6 => AxisState::EncoderIndexSearch,
            7 => AxisState::EncoderOffsetCalibration,
            8 => AxisState::ClosedLoopControl,
            9 => AxisState::LockinSpin,
            10 => AxisState::EncoderDirFind,
            11 => AxisState::Homing,
            12 => AxisState::EncoderHallPolarityCalibration,
            13 => AxisState::EncoderHallPhaseCalibration,
            14 => AxisState::AnticoggingCalibration,
            _ => AxisState::Undefined,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    VoltageControl = 0,
    TorqueControl = 1,
    VelocityControl = 2,
    PositionControl = 3,
}

impl From<u32> for ControlMode {
    fn from(value: u32) -> Self {
        match value {
            0 => ControlMode::VoltageControl,
            1 => ControlMode::TorqueControl,
            2 => ControlMode::VelocityControl,
            3 => ControlMode::PositionControl,
            _ => ControlMode::VoltageControl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Inactive = 0,
    Passthrough = 1,
    VelRamp = 2,
    PosFilter = 3,
    TrapTraj = 5,
    TorqueRamp = 6,
    Mirror = 7,
    Tuning = 8,
}

impl From<u32> for InputMode {
    fn from(value: u32) -> Self {
        match value {
            0 => InputMode::Inactive,
            1 => InputMode::Passthrough,
            2 => InputMode::VelRamp,
            3 => InputMode::PosFilter,
            5 => InputMode::TrapTraj,
            6 => InputMode::TorqueRamp,
            7 => InputMode::Mirror,
            8 => InputMode::Tuning,
            _ => InputMode::Inactive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ODriveError {
    None = 0,
    Initializing = 0x1,
    SystemLevel = 0x2,
    TimingError = 0x4,
    MissingEstimate = 0x8,
    BadConfig = 0x10,
    DrvFault = 0x20,
    MissingInput = 0x40,
    DcBusOverVoltage = 0x100,
    DcBusUnderVoltage = 0x200,
    DcBusOverCurrent = 0x400,
    DcBusOverRegenCurrent = 0x800,
    CurrentLimitViolation = 0x1000,
    MotorOverTemp = 0x2000,
    InverterOverTemp = 0x4000,
    VelocityLimitViolation = 0x8000,
    PositionLimitViolation = 0x10000,
    WatchdogTimerExpired = 0x100000,
    EstopRequested = 0x200000,
    SpinoutDetected = 0x400000,
    BrakeResistorDisarmed = 0x800000,
    ThermistorDisconnected = 0x1000000,
    CalibrationError = 0x10000000,
}

impl ODriveError {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut errors = Vec::new();
        if bits & 0x1 != 0 { errors.push(Self::Initializing); }
        if bits & 0x2 != 0 { errors.push(Self::SystemLevel); }
        if bits & 0x4 != 0 { errors.push(Self::TimingError); }
        if bits & 0x8 != 0 { errors.push(Self::MissingEstimate); }
        if bits & 0x10 != 0 { errors.push(Self::BadConfig); }
        if bits & 0x20 != 0 { errors.push(Self::DrvFault); }
        if bits & 0x40 != 0 { errors.push(Self::MissingInput); }
        if bits & 0x100 != 0 { errors.push(Self::DcBusOverVoltage); }
        if bits & 0x200 != 0 { errors.push(Self::DcBusUnderVoltage); }
        if bits & 0x400 != 0 { errors.push(Self::DcBusOverCurrent); }
        if bits & 0x800 != 0 { errors.push(Self::DcBusOverRegenCurrent); }
        if bits & 0x1000 != 0 { errors.push(Self::CurrentLimitViolation); }
        if bits & 0x2000 != 0 { errors.push(Self::MotorOverTemp); }
        if bits & 0x4000 != 0 { errors.push(Self::InverterOverTemp); }
        if bits & 0x8000 != 0 { errors.push(Self::VelocityLimitViolation); }
        if bits & 0x10000 != 0 { errors.push(Self::PositionLimitViolation); }
        if bits & 0x100000 != 0 { errors.push(Self::WatchdogTimerExpired); }
        if bits & 0x200000 != 0 { errors.push(Self::EstopRequested); }
        if bits & 0x400000 != 0 { errors.push(Self::SpinoutDetected); }
        if bits & 0x800000 != 0 { errors.push(Self::BrakeResistorDisarmed); }
        if bits & 0x1000000 != 0 { errors.push(Self::ThermistorDisconnected); }
        if bits & 0x10000000 != 0 { errors.push(Self::CalibrationError); }
        errors
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureResult {
    Success = 0,
    Busy = 1,
    Cancelled = 2,
    Disarmed = 3,
    NoResponse = 4,
    PolePairCprMismatch = 5,
    PhaseResistanceOutOfRange = 6,
    PhaseInductanceOutOfRange = 7,
    UnbalancedPhases = 8,
    InvalidMotorType = 9,
    IllegalHallState = 10,
    Timeout = 11,
    HomingWithoutEndstop = 12,
    InvalidState = 13,
    NotCalibrated = 14,
    NotConverging = 15,
}

impl From<u8> for ProcedureResult {
    fn from(value: u8) -> Self {
        match value {
            0 => ProcedureResult::Success,
            1 => ProcedureResult::Busy,
            2 => ProcedureResult::Cancelled,
            3 => ProcedureResult::Disarmed,
            4 => ProcedureResult::NoResponse,
            5 => ProcedureResult::PolePairCprMismatch,
            6 => ProcedureResult::PhaseResistanceOutOfRange,
            7 => ProcedureResult::PhaseInductanceOutOfRange,
            8 => ProcedureResult::UnbalancedPhases,
            9 => ProcedureResult::InvalidMotorType,
            10 => ProcedureResult::IllegalHallState,
            11 => ProcedureResult::Timeout,
            12 => ProcedureResult::HomingWithoutEndstop,
            13 => ProcedureResult::InvalidState,
            14 => ProcedureResult::NotCalibrated,
            15 => ProcedureResult::NotConverging,
            _ => ProcedureResult::Success,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTypes {
    Bool,
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    Float,
}

impl ValueTypes {
    pub fn byte_size(&self) -> usize {
        match self {
            ValueTypes::Bool | ValueTypes::Uint8 | ValueTypes::Int8 => 1,
            ValueTypes::Uint16 | ValueTypes::Int16 => 2,
            ValueTypes::Uint32 | ValueTypes::Int32 | ValueTypes::Float => 4,
            ValueTypes::Uint64 | ValueTypes::Int64 => 8,
        }
    }
}
