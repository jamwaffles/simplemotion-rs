mod faults;
mod parameters;
mod status;
mod statuscode;

pub use faults::Faults;
pub use parameters::ControlMode;
use parameters::Parameter;
use simplemotion_sys::{
    getCumulativeStatus, resetCumulativeStatus, smCloseBus, smOpenBus, smRead1Parameter,
    smSetBaudrate, smSetParameter, smSetTimeout,
};
pub use status::Status;
pub use statuscode::StatusCode;
use std::num::TryFromIntError;
use std::{convert::TryInto, ffi::CString};

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

    /// Encoder counts per revolution.
    ///
    /// This value is multiplied by 4 to account for quadrature encoders.
    encoder_counts: f64,

    /// Read from `[CVL]`
    velocity_limit: f64,

    /// Read from `[MUL]`
    input_mul: f64,

    /// Read from `[DIV]`
    input_div: f64,

    /// Device path, e.g. `/dev/ttyUSB0`.
    device: String,
}

impl Argon {
    /// Attempt to connect to an Argon drive at the given device and address.
    pub fn connect(device: &str, address: u8) -> Result<Self, Error> {
        log::debug!("Open {}", device);

        // Must be before bus open, must be between 1 and 5000ms
        unsafe { smSetTimeout(100) };

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
            velocity_limit: 0.0,
            input_mul: 0.0,
            input_div: 0.0,
            device: device.to_string(),
        };

        // _self.set_parameter(Parameter::BusSpeed, 115200)?;
        // _self.reconnect()?;

        _self.pid_freq = _self.read_parameter(Parameter::PIDFrequency)?.into();
        _self.encoder_counts = f64::from(_self.read_parameter(Parameter::EncoderPpr)?) * 4.0;
        _self.velocity_limit = f64::from(_self.read_parameter(Parameter::VelocityLimit)?);
        _self.input_mul = f64::from(_self.read_parameter(Parameter::InputMul)?);
        _self.input_div = f64::from(_self.read_parameter(Parameter::InputDiv)?);

        log::debug!("Initialised: {:#?}", _self);

        Ok(_self)
    }

    /// Close and reopen connection to the drive.
    pub fn reconnect(&mut self) -> Result<(), Error> {
        // Close bus. We'll ignore any errors here.
        let result = unsafe { smCloseBus(self.bus_handle) };

        log::debug!("Closing bus, status {}", result);

        // unsafe { smSetBaudrate(115200) };

        let bus_handle = {
            let device = CString::new(self.device.clone())
                .expect("Device name could not be converted to a valid C string");

            let handle: i64 = unsafe { smOpenBus(device.as_ptr()) };

            if handle >= 0 {
                Ok(handle)
            } else {
                Err(Error::OpenFailed(handle.into()))
            }
        }?;

        log::info!("--> Reconnected");

        self.bus_handle = bus_handle;

        Ok(())
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
        let value = value.into();

        let result: StatusCode =
            unsafe { smSetParameter(self.bus_handle, self.address, parameter as i16, value) }
                .into();

        log::trace!(
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
    ///
    /// Note that this is returned as an `i32` however some values are shorter than the 4 bytes
    /// consumed by it. Converting to bytes then into the correct type may be required.
    fn read_parameter(&self, parameter: Parameter) -> Result<i32, Error> {
        // TODO: Check that bus is open

        let mut output = 0;

        let result: StatusCode = unsafe {
            smRead1Parameter(self.bus_handle, self.address, parameter as i16, &mut output)
        }
        .into();

        log::trace!("Read parameter {:?}. Got value {:?}", parameter, output);

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

    pub fn set_homing_complete(&self) -> Result<(), Error> {
        // TODO: Can I move this into `home()` and call it before all the other methods?
        self.set_parameter(Parameter::HomingControl, 0)
    }

    /// Get the current actual raw velocity.
    ///
    /// IIUC, this is the number of encoder counts per PID loop period which is usually/always
    /// 2500Hz or 400uS.
    fn velocity_raw(&self) -> Result<i16, Error> {
        let raw = self.read_parameter(Parameter::ActualVelocity)?;

        let bytes = raw.to_le_bytes();

        let value_bytes = bytes[0..2].try_into().unwrap();

        Ok(i16::from_le_bytes(value_bytes))
    }

    /// Get velocity (RPS) setpoint.
    pub fn setpoint_rps(&self) -> Result<f64, Error> {
        let feedback: f64 = self.absolute_setpoint()?.into();

        // FIXME: input_div just _happens_ to work - find out why/where/if it's correct
        let rps = (feedback * self.pid_freq) / self.encoder_counts() / self.input_div;

        log::trace!("Feedback RPS {}", rps);

        Ok(rps)
    }

    /// Scale RPS value to drive setpoint.
    fn rps_to_setpoint(&self, rps: f64) -> f64 {
        let max_velocity =
            (self.velocity_limit * self.pid_freq) / (self.encoder_counts() * self.input_div);

        (rps / max_velocity) * self.velocity_limit * (self.input_mul / self.input_div)
    }

    /// Set the velocity by RPS value.
    pub fn set_velocity_rps(&self, rps: f64) -> Result<(), Error> {
        let setpoint = self.rps_to_setpoint(rps);

        self.set_absolute_setpoint(setpoint.round() as i32)
    }

    /// Get the actual RPS (Revolutions Per Second).
    pub fn velocity_rps(&mut self) -> Result<f64, Error> {
        let feedback: f64 = self.velocity_raw()?.into();

        let rps = feedback / (self.encoder_counts() / self.pid_freq);

        Ok(rps)
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
