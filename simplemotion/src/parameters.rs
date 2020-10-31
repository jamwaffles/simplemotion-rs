use simplemotion_sys::{
    CM_NONE, CM_POSITION, CM_TORQUE, CM_VELOCITY, SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE,
    SMP_FAULTS, SMP_STATUS,
};

/// Non-exhaustive list of drive parameters.
#[derive(Debug, Copy, Clone)]
pub enum Parameter {
    AbsoluteSetpoint = SMP_ABSOLUTE_SETPOINT as isize,
    Faults = SMP_FAULTS as isize,
    Status = SMP_STATUS as isize,
    ControlMode = SMP_CONTROL_MODE as isize,
}

/// Control mode.
pub enum ControlMode {
    Torque = CM_TORQUE as isize,
    Velocity = CM_VELOCITY as isize,
    Position = CM_POSITION as isize,
    None = CM_NONE as isize,
}
