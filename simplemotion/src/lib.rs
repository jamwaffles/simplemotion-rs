mod faults;
mod moving_average;
mod parameters;
mod status;
mod statuscode;

pub use faults::Faults;
use moving_average::MovingAverage;
pub use parameters::ControlMode;
use parameters::Parameter;
use simplemotion_sys::{
    getCumulativeStatus, resetCumulativeStatus, smCloseBus, smOpenBus, smRead1Parameter,
    smSetParameter, CM_POSITION, CM_VELOCITY, SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE, SMP_FAULTS,
    SM_ERR_BUS, SM_ERR_COMMUNICATION, SM_ERR_LENGTH, SM_ERR_NODEVICE, SM_ERR_PARAMETER, SM_NONE,
    SM_OK,
};
pub use status::Status;
pub use statuscode::StatusCode;
use std::ffi::CString;
use std::{convert::TryFrom, num::TryFromIntError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Bus open failed. Check the device name and that it is not locked by another program. Code: {0:?}"
    )]
    OpenFailed(StatusCode),

    #[error("Could not set drive parameter {0:?} to {1}. Code: {2:?}")]
    SetParameter(Parameter, i32, StatusCode),

    #[error("Could not read drive parameter {0:?}. Code: {1:?}")]
    ReadParameter(Parameter, StatusCode),

    #[error("Failed to read drive status. Code: {0:?}")]
    GetStatus(StatusCode),

    #[error("Failed to reset drive status. Code: {0:?}")]
    ResetStatus(StatusCode),

    #[error("Value conversion failed: {0:?}")]
    ValueConversion(TryFromIntError),
}

#[derive(Debug)]
pub struct Argon {
    address: u8,
    bus_handle: i64,
    pid_freq: f64,

    /// Counts per revolution.
    ///
    /// For quadrature encoders this is 4x the PPR.
    encoder_counts: f64,

    velocity_rps_avg: MovingAverage,
}

impl Argon {
    /// Attempt to connect to an Argon drive at the given device and address.
    pub fn connect(device: &str, address: u8) -> Result<Self, Error> {
        log::debug!("Open {}", device);

        let bus_handle = {
            let device = CString::new(device)
                .expect("Device name could not be converted to a valid C string");

            let handle: i64 = unsafe { smOpenBus(device.as_ptr()) };

            if handle >= 0 {
                Ok(handle)
            } else {
                Err(Error::OpenFailed(handle.into()))
            }
        }?;

        log::debug!("Bus {}", bus_handle);

        let bus_status = {
            let result = unsafe { getCumulativeStatus(bus_handle) };

            Ok(result)
        }?;

        log::debug!("Bus status {}", bus_status);

        let mut _self = Self {
            address,
            bus_handle,
            pid_freq: 0.0,
            encoder_counts: 0.0,
            velocity_rps_avg: MovingAverage::new(10),
        };

        _self.pid_freq = _self.read_parameter(Parameter::PIDFrequency)?.into();
        _self.encoder_counts = f64::from(_self.read_parameter(Parameter::EncoderPpr)?) * 4.0;

        Ok(_self)
    }

    /// Encoder counts.
    pub fn encoder_counts(&self) -> f64 {
        self.encoder_counts
    }

    /// Set a parameter in the drive.
    fn set_parameter<V>(&self, parameter: Parameter, value: V) -> Result<(), Error>
    where
        V: Into<i32>,
    {
        // TODO: Check that bus is open

        let value = value.into();

        let result: StatusCode =
            unsafe { smSetParameter(self.bus_handle, self.address, parameter as i16, value) }
                .into();

        log::debug!(
            "Set parameter {:?} to {}. Result: {:?}",
            parameter,
            value,
            result
        );

        if result.is_err() {
            Err(Error::SetParameter(parameter, value, result))
        } else {
            Ok(())
        }
    }

    /// Read a parameter in the drive.
    fn read_parameter(&self, parameter: Parameter) -> Result<i32, Error> {
        // TODO: Check that bus is open

        let mut output = 0;

        let result: StatusCode = unsafe {
            smRead1Parameter(self.bus_handle, self.address, parameter as i16, &mut output)
        }
        .into();

        log::debug!("Read parameter {:?}. Got value {:?}", parameter, output);

        if result.is_ok() {
            Ok(output)
        } else {
            Err(Error::ReadParameter(parameter, result))
        }
    }

    /// Attempt to clear drive faults.
    ///
    /// The drive status should be checked after this method is called, before other actions are
    /// performed.
    pub fn clear_faults(&mut self) -> Result<(), Error> {
        self.set_parameter(Parameter::Faults, 0)?;

        let result = unsafe { resetCumulativeStatus(self.bus_handle) };

        if result == StatusCode::Ok as i32 {
            Ok(())
        } else {
            Err(Error::ResetStatus(result.into()))
        }
    }

    /// Get cumulative drive status since last reset.
    pub fn status(&self) -> Result<Status, Error> {
        // let result = unsafe { getCumulativeStatus(self.bus_handle) };
        let result = self.read_parameter(Parameter::Status)?;

        log::trace!("Raw status result {} {:0b}", result, result);

        Ok(Status::from(result as u32))
    }

    /// Get drive faults.
    pub fn faults(&self) -> Result<Faults, Error> {
        let param = self.read_parameter(Parameter::Faults)?;

        Ok(Faults::from(param as u32))
    }

    /// Read drive fault status.
    pub fn is_online(&self) -> Result<bool, Error> {
        Ok(self.status()?.run)
    }

    /// Set the raw setpoint.
    fn set_absolute_setpoint(&self, setpoint: i32) -> Result<(), Error> {
        self.set_parameter(Parameter::AbsoluteSetpoint, setpoint)
    }

    /// Get current setpoint value.
    fn absolute_setpoint(&self) -> Result<i32, Error> {
        self.read_parameter(Parameter::AbsoluteSetpoint)
    }

    /// Set control mode.
    pub fn set_control_mode(&self, mode: ControlMode) -> Result<(), Error> {
        self.set_parameter(Parameter::ControlMode, mode as i32)
    }

    /// Put the drive into position mode and search for the home (index) pulse.
    ///
    /// A non-zero offset in degrees can be provided to position the shaft at an arbitrary angle
    /// relative to the index. Homing direction is positive.
    pub fn home(&self, offset: f64) -> Result<(), Error> {
        let offset_counts = (self.encoder_counts() / 360.0) * offset;

        self.set_control_mode(ControlMode::Position)?;
        self.set_parameter(
            Parameter::TrajPlannerHomingOffset,
            offset_counts.round() as i32,
        )?;
        self.set_parameter(Parameter::HomingControl, 1)
    }

    /// Get the current actual raw velocity.
    ///
    /// IIUC, this is the number of encoder counts per PID loop period which is usually/always
    /// 2500Hz or 400uS.
    fn velocity_raw(&self) -> Result<i32, Error> {
        self.read_parameter(Parameter::ActualVelocity)
    }

    /// Get velocity (RPS) setpoint.
    pub fn setpoint_rps(&self) -> Result<f64, Error> {
        let feedback: f64 = self.absolute_setpoint()?.into();

        let rps = (feedback * 100.0) / self.encoder_counts();

        log::debug!("Set RPS to {}", rps);

        Ok(rps)
    }

    /// Set the velocity by RPS value.
    pub fn set_velocity_rps(&self, rps: f64) -> Result<(), Error> {
        let setpoint = (rps * self.encoder_counts()) / 100.0;

        self.set_absolute_setpoint(setpoint.round() as i32)
    }

    /// Get the smoothed RPS (Revolutions Per Second).
    pub fn velocity_rps(&mut self) -> Result<f64, Error> {
        let feedback: f64 = self.velocity_raw()?.into();

        let rps = feedback / (self.encoder_counts() / self.pid_freq);

        Ok(self.velocity_rps_avg.feed(rps))
    }
}

impl Drop for Argon {
    fn drop(&mut self) {
        log::debug!("Close Argon connection");

        let result: StatusCode = unsafe { smCloseBus(self.bus_handle) }.into();

        if result.is_err() {
            log::error!("Failed to close bus handle: {:?}", result);
        }
    }
}
