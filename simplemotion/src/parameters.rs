use simplemotion_sys::{
    CM_NONE, CM_POSITION, CM_TORQUE, CM_VELOCITY, SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE,
    SMP_FAULTS, SMP_HOMING_CONTROL, SMP_STATUS, SMP_TRAJ_PLANNER_HOMING_OFFSET,
};
use std::fmt;

/// Non-exhaustive list of drive parameters.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Parameter {
    AbsoluteSetpoint = SMP_ABSOLUTE_SETPOINT as isize,
    Faults = SMP_FAULTS as isize,
    Status = SMP_STATUS as isize,
    ControlMode = SMP_CONTROL_MODE as isize,
    HomingControl = SMP_HOMING_CONTROL as isize,
    TrajPlannerHomingOffset = SMP_TRAJ_PLANNER_HOMING_OFFSET as isize,
}

/// Control mode.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ControlMode {
    Torque = CM_TORQUE as isize,
    Velocity = CM_VELOCITY as isize,
    Position = CM_POSITION as isize,
    None = CM_NONE as isize,
}

impl fmt::Display for ControlMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Torque => f.write_str("Torque"),
            Self::Velocity => f.write_str("Velocity"),
            Self::Position => f.write_str("Position"),
            Self::None => f.write_str("None"),
        }
    }
}
