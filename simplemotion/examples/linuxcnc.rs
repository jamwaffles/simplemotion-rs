use linuxcnc_hal::{
    error::PinRegisterError, hal_pin::InputPin, prelude::*, HalComponent, RegisterResources,
    Resources,
};
use simplemotion::{Argon, ControlMode};
use std::{error::Error, thread, time::Duration};

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

fn main() -> Result<(), Box<dyn Error>> {
    let device = std::env::args().nth(1).expect("Device name/path required");
    let address: u8 = std::env::args()
        .nth(2)
        .expect("Device address is required")
        .parse()
        .expect("Device address must be a number from 1 - 255");

    pretty_env_logger::init();

    let argon = Argon::connect(&device, address)?;

    let comp: HalComponent<Pins> = HalComponent::new("argon")?;

    // Get a reference to the `Pins` struct
    let pins = comp.resources();

    log::debug!("Pins: {:?}", pins);

    let mut current_speed = 0.0f64;
    let mut current_pos = 0.0f64;
    let mut current_mode = ControlMode::Velocity;

    argon.set_control_mode(current_mode)?;

    // Main control loop
    while !comp.should_exit() {
        let new_mode = if *pins.orient_enable.value()? {
            ControlMode::Position
        } else {
            ControlMode::Velocity
        };

        if new_mode != current_mode {
            log::debug!("Changing control mode to {}", new_mode);

            argon.set_control_mode(new_mode)?;
            current_mode = new_mode;
        }

        match current_mode {
            ControlMode::Velocity => {
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

                    argon.set_absolute_setpoint(new_speed as i32)?;
                }
            }
            ControlMode::Position => {
                // SAFETEY: Default to current pos (no movement) if error occurred
                let new_pos = *pins.orient_angle.value().unwrap_or_else(|e| {
                    log::error!(
                        "Failed to get spindle speed value: {}. Defaulting to 0.0",
                        e
                    );

                    &0.0
                });

                log::debug!("{:?}", argon.status()?.homing);

                if new_pos != current_pos {
                    current_pos = new_pos;

                    // log::info!("Changing setpoint to {}", new_pos);

                    // argon.set_absolute_setpoint(new_pos as i32)?;
                    log::debug!("Homing");
                    argon.home(0)?;
                }
            }
            mode => panic!("Unsupported control mode {}", mode),
        }

        thread::sleep(Duration::from_millis(50));
    }

    // Bare minimum safe state on shutdown.
    // FIXME: Check if I need to set anything else.
    argon.set_absolute_setpoint(0)?;

    Ok(())
}
