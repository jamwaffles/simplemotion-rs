use simplemotion_sys::{
    STAT_BRAKING, STAT_ENABLED, STAT_FAULTSTOP, STAT_FERROR_RECOVERY, STAT_FERROR_WARNING,
    STAT_HOMING, STAT_INITIALIZED, STAT_PERMANENT_STOP, STAT_QUICK_STOP_ACTIVE, STAT_RUN,
    STAT_SAFE_TORQUE_MODE_ACTIVE, STAT_SERVO_READY, STAT_STANDBY, STAT_STANDING_STILL,
    STAT_STO_ACTIVE, STAT_TARGET_REACHED, STAT_VOLTAGES_OK,
};

#[derive(Debug)]
pub struct Status {
    pub target_reached: bool,
    pub ferror_recovery: bool,
    pub run: bool,
    pub enabled: bool,
    pub faultstop: bool,
    pub ferror_warning: bool,
    pub sto_active: bool,
    pub servo_ready: bool,
    pub braking: bool,
    pub homing: bool,
    pub initialized: bool,
    pub voltages_ok: bool,
    pub permanent_stop: bool,
    pub standing_still: bool,
    pub quick_stop_active: bool,
    pub safe_torque_mode_active: bool,
    pub standby: bool,
}

impl From<u32> for Status {
    fn from(other: u32) -> Self {
        Self {
            target_reached: other & STAT_TARGET_REACHED > 0,
            ferror_recovery: other & STAT_FERROR_RECOVERY > 0,
            run: other & STAT_RUN > 0,
            enabled: other & STAT_ENABLED > 0,
            faultstop: other & STAT_FAULTSTOP > 0,
            ferror_warning: other & STAT_FERROR_WARNING > 0,
            sto_active: other & STAT_STO_ACTIVE > 0,
            servo_ready: other & STAT_SERVO_READY > 0,
            braking: other & STAT_BRAKING > 0,
            homing: other & STAT_HOMING > 0,
            initialized: other & STAT_INITIALIZED > 0,
            voltages_ok: other & STAT_VOLTAGES_OK > 0,
            permanent_stop: other & STAT_PERMANENT_STOP > 0,
            standing_still: other & STAT_STANDING_STILL > 0,
            quick_stop_active: other & STAT_QUICK_STOP_ACTIVE > 0,
            safe_torque_mode_active: other & STAT_SAFE_TORQUE_MODE_ACTIVE > 0,
            standby: other & STAT_STANDBY > 0,
        }
    }
}
