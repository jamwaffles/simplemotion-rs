use simplemotion_sys::{SMP_ABSOLUTE_SETPOINT, SMP_FAULTS, SMP_STATUS};

/// Non-exhaustive list of drive parameters.
#[derive(Debug, Copy, Clone)]
pub enum Parameter {
    AbsoluteSetpoint = SMP_ABSOLUTE_SETPOINT as isize,
    Faults = SMP_FAULTS as isize,
    Status = SMP_STATUS as isize,
}
