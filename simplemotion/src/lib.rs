mod faults;
mod parameters;
mod status;
mod statuscode;

pub use faults::Faults;
pub use parameters::ControlMode;
pub use status::Status;
pub use statuscode::StatusCode;

use parameters::Parameter;
use simplemotion_sys::{
    getCumulativeStatus, resetCumulativeStatus, smCloseBus, smOpenBus, smRead1Parameter,
    smSetParameter, CM_POSITION, CM_VELOCITY, SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE, SMP_FAULTS,
    SM_ERR_BUS, SM_ERR_COMMUNICATION, SM_ERR_LENGTH, SM_ERR_NODEVICE, SM_ERR_PARAMETER, SM_NONE,
    SM_OK,
};
use std::ffi::CString;

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
}

pub struct Argon {
    address: u8,
    bus_handle: i64,
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

        Ok(Self {
            address,
            bus_handle,
        })
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
        let result = unsafe { getCumulativeStatus(self.bus_handle) };

        if result == StatusCode::Ok as i32 {
            Ok(Status::from(result as u32))
        } else {
            Err(Error::GetStatus(result.into()))
        }
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
    pub fn set_absolute_setpoint(&self, setpoint: i32) -> Result<(), Error> {
        self.set_parameter(Parameter::AbsoluteSetpoint, setpoint)
    }

    /// Set control mode.
    pub fn set_control_mode(&self, mode: ControlMode) -> Result<(), Error> {
        self.set_parameter(Parameter::ControlMode, mode as i32)
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
