use linuxcnc_hal::{
    error::PinRegisterError,
    hal_pin::{InputPin, OutputPin},
    prelude::*,
    HalComponent, RegisterResources, Resources,
};
use simplemotion_sys::{
    getCumulativeStatus, smCloseBus, smOpenBus, smSetParameter, CM_POSITION, CM_VELOCITY,
    SMP_CONTROL_MODE, SMP_FAULTS,
};
use std::ffi::CString;
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, thiserror::Error)]
enum Errors {
    #[error("Bus open failed")]
    OpenFailed(i64),
}

// fn main() -> Result<(), Errors> {
//     let device = std::env::args().nth(1).expect("Device name/path required");
//     let arg = std::env::args().nth(2).expect("Velocity mode required");

//     let arg = match arg.as_str() {
//         "pos" | "position" => CM_POSITION,
//         "vel" | "velocity" => CM_VELOCITY,
//         _ => panic!("Expected one of 'pos' or 'vel'"),
//     };

//     println!("Open {}", device);

//     let bus = {
//         let device = CString::new(device).expect("Failed to convert device to CString");

//         let handle = unsafe { smOpenBus(device.as_ptr()) };

//         if handle >= 0 {
//             Ok(handle)
//         } else {
//             Err(Errors::OpenFailed(handle))
//         }
//     }?;

//     println!("Bus {}", bus);

//     let bus_status = {
//         let result = unsafe { getCumulativeStatus(bus) };

//         Ok(result)
//     }?;

//     println!("Bus status {}", bus_status);

//     let result = unsafe { smSetParameter(bus, 1, SMP_FAULTS as i16, 0) };

//     println!("Reset done with {}", result);

//     let result = unsafe { smSetParameter(bus, 1, SMP_CONTROL_MODE as i16, arg as i32) };

//     println!("Control mode set with result {}", result);

//     unsafe { smCloseBus(bus) };

//     Ok(())
// }

struct Pins {
    control_mode: InputPin<u32>,
}

impl Resources for Pins {
    type RegisterError = PinRegisterError;

    fn register_resources(comp: &RegisterResources) -> Result<Self, Self::RegisterError> {
        Ok(Pins {
            control_mode: comp.register_pin("control-mode")?,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Args {:?}", std::env::args());

    pretty_env_logger::init();

    let comp: HalComponent<Pins> = HalComponent::new("argon")?;

    // Get a reference to the `Pins` struct
    let pins = comp.resources();

    let start = Instant::now();

    // Main control loop
    while !comp.should_exit() {
        log::info!("Control mode {:?}", pins.control_mode);

        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}
