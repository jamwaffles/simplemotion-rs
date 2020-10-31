use simplemotion_sys::{
    FLT_COMMUNICATION, FLT_CONFIG, FLT_ENCODER, FLT_FOLLOWERROR, FLT_HARDWARE, FLT_HOST_COMM_ERROR,
    FLT_INIT, FLT_MOTION, FLT_OVERCURRENT, FLT_OVERTEMP, FLT_OVERVELOCITY, FLT_OVERVOLTAGE,
    FLT_PROGRAM_OR_MEM, FLT_PSTAGE_FORCED_OFF, FLT_RANGE, FLT_UNDERVOLTAGE,
};

/// Drive faults.
#[derive(Debug)]
pub struct Faults {
    pub followerror: bool,
    pub overcurrent: bool,
    pub communication: bool,
    pub encoder: bool,
    pub overtemp: bool,
    pub undervoltage: bool,
    pub overvoltage: bool,
    pub program_or_mem: bool,
    pub hardware: bool,
    pub overvelocity: bool,
    pub init: bool,
    pub motion: bool,
    pub range: bool,
    pub pstage_forced_off: bool,
    pub host_comm_error: bool,
    pub config: bool,
}

impl From<u32> for Faults {
    fn from(other: u32) -> Self {
        Self {
            followerror: other & FLT_FOLLOWERROR > 0,
            overcurrent: other & FLT_OVERCURRENT > 0,
            communication: other & FLT_COMMUNICATION > 0,
            encoder: other & FLT_ENCODER > 0,
            overtemp: other & FLT_OVERTEMP > 0,
            undervoltage: other & FLT_UNDERVOLTAGE > 0,
            overvoltage: other & FLT_OVERVOLTAGE > 0,
            program_or_mem: other & FLT_PROGRAM_OR_MEM > 0,
            hardware: other & FLT_HARDWARE > 0,
            overvelocity: other & FLT_OVERVELOCITY > 0,
            init: other & FLT_INIT > 0,
            motion: other & FLT_MOTION > 0,
            range: other & FLT_RANGE > 0,
            pstage_forced_off: other & FLT_PSTAGE_FORCED_OFF > 0,
            host_comm_error: other & FLT_HOST_COMM_ERROR > 0,
            config: other & FLT_CONFIG > 0,
        }
    }
}
