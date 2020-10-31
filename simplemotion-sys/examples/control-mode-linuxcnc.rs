use linuxcnc_hal::{
    error::PinRegisterError,
    hal_pin::{InputPin, OutputPin},
    prelude::*,
    HalComponent, RegisterResources, Resources,
};
use simplemotion_sys::{
    getCumulativeStatus, smCloseBus, smOpenBus, smSetParameter, CM_POSITION, CM_VELOCITY,
    SMP_ABSOLUTE_SETPOINT, SMP_CONTROL_MODE, SMP_FAULTS,
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

#[derive(Debug)]
struct Pins {
    orient_enable: InputPin<bool>,
    orient_angle: InputPin<f64>,
    spindle_speed_rpm: InputPin<f64>,
}

impl Resources for Pins {
    type RegisterError = PinRegisterError;

    fn register_resources(comp: &RegisterResources) -> Result<Self, Self::RegisterError> {
        Ok(Pins {
            orient_enable: comp.register_pin("orient-enable")?,
            orient_angle: comp.register_pin("orient-angle")?,
            spindle_speed_rpm: comp.register_pin("spindle-speed-rpm")?,
        })
    }
}

fn init_argon(device: &str) -> Result<i64, Errors> {
    log::debug!("Open {}", device);

    let bus = {
        let device = CString::new(device).expect("Failed to convert device to CString");

        let handle = unsafe { smOpenBus(device.as_ptr()) };

        if handle >= 0 {
            Ok(handle)
        } else {
            Err(Errors::OpenFailed(handle))
        }
    }?;

    log::debug!("Bus {}", bus);

    let bus_status = {
        let result = unsafe { getCumulativeStatus(bus) };

        Ok(result)
    }?;

    log::debug!("Bus status {}", bus_status);

    let result = unsafe { smSetParameter(bus, 1, SMP_FAULTS as i16, 0) };

    log::debug!("Reset done with {}", result);

    Ok(bus)
}

fn main() -> Result<(), Box<dyn Error>> {
    let device = std::env::args().nth(1).expect("Device name/path required");

    let bus = init_argon(&device)?;

    pretty_env_logger::init();

    let comp: HalComponent<Pins> = HalComponent::new("argon")?;

    // Get a reference to the `Pins` struct
    let pins = comp.resources();

    log::debug!("Pins: {:?}", pins);

    let start = Instant::now();

    let mut current_speed = 0.0f64;

    // Main control loop
    while !comp.should_exit() {
        // log::info!(
        //     "Orient enable: {:?}, angle {:?}, speed {:?}",
        //     pins.orient_enable.value(),
        //     pins.orient_angle.value(),
        //     pins.spindle_speed_rpm.value()
        // );

        // SAFETEY: Default to 0 RPM if some error occurred.
        let new_speed = *pins.spindle_speed_rpm.value().unwrap_or_else(|e| {
            log::error!(
                "Failed to get spindle speed value: {}. Defaulting to 0.0",
                e
            );

            &0.0
        });

        if new_speed != current_speed {
            current_speed = new_speed;

            log::info!("Changing setpoint to {} RPM", new_speed);

            let result =
                unsafe { smSetParameter(bus, 1, SMP_ABSOLUTE_SETPOINT as i16, new_speed as i32) };

            log::debug!("Setpoint set with result {}", result);
        }

        thread::sleep(Duration::from_millis(10));
    }

    unsafe { smCloseBus(bus) };

    Ok(())
}
