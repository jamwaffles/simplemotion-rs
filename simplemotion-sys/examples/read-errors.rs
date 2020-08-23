use simplemotion_sys::{
    getCumulativeStatus, smCloseBus, smOpenBus, smRead1Parameter, FLT_COMMUNICATION, FLT_CONFIG,
    FLT_ENCODER, FLT_FOLLOWERROR, FLT_HARDWARE, FLT_HOST_COMM_ERROR, FLT_INIT, FLT_MOTION,
    FLT_OVERCURRENT, FLT_OVERTEMP, FLT_OVERVELOCITY, FLT_OVERVOLTAGE, FLT_PROGRAM_OR_MEM,
    FLT_PSTAGE_FORCED_OFF, FLT_RANGE, FLT_UNDERVOLTAGE, SMP_FAULTS, SMP_STATUS, SM_OK,
    STAT_BRAKING, STAT_ENABLED, STAT_FAULTSTOP, STAT_FERROR_RECOVERY, STAT_FERROR_WARNING,
    STAT_HOMING, STAT_INITIALIZED, STAT_PERMANENT_STOP, STAT_QUICK_STOP_ACTIVE, STAT_RUN,
    STAT_SAFE_TORQUE_MODE_ACTIVE, STAT_SERVO_READY, STAT_STANDBY, STAT_STANDING_STILL,
    STAT_STO_ACTIVE, STAT_TARGET_REACHED, STAT_VOLTAGES_OK,
};
use std::ffi::CString;

#[derive(Debug, thiserror::Error)]
enum Errors {
    #[error("Bus open failed")]
    OpenFailed(i64),

    #[error("Read 1 param failed")]
    Read1,
}

fn main() -> Result<(), Errors> {
    let device = std::env::args().nth(1).expect("Device name/path required");

    println!("Open {}", device);

    let bus = {
        let device = CString::new(device).expect("Failed to convert device to CString");

        let handle = unsafe { smOpenBus(device.as_ptr()) };

        if handle >= 0 {
            Ok(handle)
        } else {
            Err(Errors::OpenFailed(handle))
        }
    }?;

    println!("Bus {}", bus);

    let bus_status = {
        let result = unsafe { getCumulativeStatus(bus) };

        Ok(result)
    }?;
    println!("Status {}", bus_status);

    let device_status = {
        let mut s_m_device_side_comm_status = 0;

        let result = unsafe {
            smRead1Parameter(
                bus,
                1,
                // SMP_CUMULATIVE_STATUS as i16,
                SMP_STATUS as i16,
                &mut s_m_device_side_comm_status,
            )
        };

        println!(
            "Read1 result {}, comm_status {}",
            result, s_m_device_side_comm_status
        );

        if result == SM_OK as i32 {
            Ok(s_m_device_side_comm_status)
        } else {
            Err(Errors::Read1)
        }
    }?;

    let device_faults = {
        let mut s_m_device_side_comm_status = 0;

        let result = unsafe {
            smRead1Parameter(
                bus,
                1,
                // SMP_CUMULATIVE_STATUS as i16,
                SMP_FAULTS as i16,
                &mut s_m_device_side_comm_status,
            )
        };

        if result == SM_OK as i32 {
            Ok(s_m_device_side_comm_status)
        } else {
            Err(Errors::Read1)
        }
    }?;

    println!(
        "Device status {:#032b} Status \n{:#?}\nFaults\n{:#?}",
        device_status,
        Status::from(device_status as u32),
        Faults::from(device_faults as u32)
    );

    unsafe { smCloseBus(bus) };

    Ok(())
}

#[derive(Debug)]
struct Status {
    target_reached: bool,
    ferror_recovery: bool,
    run: bool,
    enabled: bool,
    faultstop: bool,
    ferror_warning: bool,
    sto_active: bool,
    servo_ready: bool,
    braking: bool,
    homing: bool,
    initialized: bool,
    voltages_ok: bool,
    permanent_stop: bool,
    standing_still: bool,
    quick_stop_active: bool,
    safe_torque_mode_active: bool,
    standby: bool,
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

#[derive(Debug)]
struct Faults {
    followerror: bool,
    overcurrent: bool,
    communication: bool,
    encoder: bool,
    overtemp: bool,
    undervoltage: bool,
    overvoltage: bool,
    program_or_mem: bool,
    hardware: bool,
    overvelocity: bool,
    init: bool,
    motion: bool,
    range: bool,
    pstage_forced_off: bool,
    host_comm_error: bool,
    config: bool,
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
