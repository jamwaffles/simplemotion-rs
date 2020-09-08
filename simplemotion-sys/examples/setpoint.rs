use simplemotion_sys::{
    getCumulativeStatus, smCloseBus, smOpenBus, smSetParameter, SMP_ABSOLUTE_SETPOINT, SMP_FAULTS,
};
use std::ffi::CString;

#[derive(Debug, thiserror::Error)]
enum Errors {
    #[error("Bus open failed")]
    OpenFailed(i64),
}

fn main() -> Result<(), Errors> {
    let device = std::env::args().nth(1).expect("Device name/path required");
    let arg = std::env::args().nth(2).expect("Expected second argument");

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

    println!("Bus status {}", bus_status);

    let result = unsafe { smSetParameter(bus, 1, SMP_FAULTS as i16, 0) };

    println!("Reset done with {}", result);

    let result =
        unsafe { smSetParameter(bus, 1, SMP_ABSOLUTE_SETPOINT as i16, arg.parse().unwrap()) };

    println!("Setpoint set with result {}", result);

    unsafe { smCloseBus(bus) };

    Ok(())
}
