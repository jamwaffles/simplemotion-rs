use simplemotion_sys::{
    getCumulativeStatus, smCloseBus, smOpenBus, smRead1Parameter, smSetParameter, CM_POSITION,
    CM_VELOCITY, SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE, SMP_FAULTS, SM_ERR_BUS,
    SM_ERR_COMMUNICATION, SM_ERR_LENGTH, SM_ERR_NODEVICE, SM_ERR_PARAMETER, SM_NONE, SM_OK,
};

/// Status.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusCode {
    None = SM_NONE as isize,
    Ok = SM_OK as isize,
    ErrNodevice = SM_ERR_NODEVICE as isize,
    ErrBus = SM_ERR_BUS as isize,
    ErrCommunication = SM_ERR_COMMUNICATION as isize,
    ErrParameter = SM_ERR_PARAMETER as isize,
    ErrLength = SM_ERR_LENGTH as isize,
    Unknown = 999,
}

impl StatusCode {
    pub fn is_ok(&self) -> bool {
        *self == Self::Ok
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

impl From<u32> for StatusCode {
    fn from(value: u32) -> Self {
        match value {
            SM_NONE => Self::None,
            SM_OK => Self::Ok,
            SM_ERR_NODEVICE => Self::ErrNodevice,
            SM_ERR_BUS => Self::ErrBus,
            SM_ERR_COMMUNICATION => Self::ErrCommunication,
            SM_ERR_PARAMETER => Self::ErrParameter,
            SM_ERR_LENGTH => Self::ErrLength,
            _ => Self::Unknown,
        }
    }
}

impl From<i64> for StatusCode {
    fn from(value: i64) -> Self {
        (value as u32).into()
    }
}

impl From<i32> for StatusCode {
    fn from(value: i32) -> Self {
        if value < 0 {
            StatusCode::Unknown
        } else {
            (value as u32).into()
        }
    }
}

impl PartialEq<i32> for StatusCode {
    fn eq(&self, other: &i32) -> bool {
        *self as i32 == *other
    }
}
