use linuxcnc_hal::{
    error::PinRegisterError, hal_pin::InputPin, hal_pin::OutputPin, prelude::*, HalComponent,
    RegisterResources, Resources,
};
use simplemotion::{Argon, ControlMode};
use std::{error::Error, thread, time::Duration};

#[derive(Debug)]
struct Pins {
    /// Whether to start orienting the spindle or not.
    orient_enable: InputPin<bool>,

    /// Orient position relative to index pulse in degrees.
    orient_angle: InputPin<f64>,

    /// Spindle speed setpoint in Revolutions Per Second.
    spindle_speed_rps: InputPin<f64>,

    /// Flag to signal that the orient is complete.
    is_oriented: OutputPin<bool>,

    /// Motor RPM.
    rpm: OutputPin<f64>,
}

impl Resources for Pins {
    type RegisterError = PinRegisterError;

    fn register_resources(comp: &RegisterResources) -> Result<Self, Self::RegisterError> {
        Ok(Pins {
            orient_enable: comp.register_pin("orient-enable")?,
            orient_angle: comp.register_pin("orient-angle")?,
            spindle_speed_rps: comp.register_pin("spindle-speed-rps")?,
            is_oriented: comp.register_pin("is-oriented")?,
            rpm: comp.register_pin("rpm")?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum State {
    SwitchToSpindle,
    Spindle,
    SwitchToOrient,
    Orienting,
    Oriented,
}

const PID_FREQ: f64 = 2500.0;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let device = std::env::args().nth(1).expect("Device name/path required");
    let address: u8 = std::env::args()
        .nth(2)
        .expect("Device address is required")
        .parse()
        .expect("Device address must be a number from 1 - 255");

    let mut argon = Argon::connect(&device, address)?;

    let comp: HalComponent<Pins> = HalComponent::new("argon")?;
    let pins = comp.resources();

    log::debug!("Pins: {:?}", pins);

    // let encoder_ppr = argon.encoder_ppr()?;
    // let pid_freq = argon.pid_frequency()? as f64;

    // log::debug!("Encoder PPR: {}, PID frequency: {}", encoder_ppr, pid_freq);

    // // Number of counts per revolution (4 x PPR as this is hardcoded to quadrature)
    // // FIXME: Support other types of encoder/resolver.
    // let encoder_counts = encoder_ppr as f64 * 4.0;

    // Initial state on startup.
    let mut state = State::SwitchToSpindle;

    // Main control loop
    while !comp.should_exit() {
        // log::trace!("State {:?}", state);

        let current_velocity_setpoint_rps = argon.setpoint_rps()?;
        let new_velocity_rps = *pins.spindle_speed_rps.value()?;
        let current_velocity_rps = argon.velocity_rps()?;

        let orient_enable = *pins.orient_enable.value()?;
        let orient_position_counts = *pins.orient_angle.value()? * argon.encoder_counts();

        log::trace!(
            "RPS: Setpoint {}, current velocity {}, new velocity {}",
            current_velocity_setpoint_rps,
            current_velocity_rps,
            new_velocity_rps
        );

        // let velocity = argon.velocity()? as f64;
        // let rps = velocity / (encoder_counts / pid_freq);
        // let rpm = rps * 60.0;

        // log::trace!(
        //     "RPS {} / {} RPM (velocity {} / (encoder counts {} / pid freq {}))",
        //     rps,
        //     rpm,
        //     velocity,
        //     encoder_counts,
        //     pid_freq
        // );

        pins.rpm.set_value(current_velocity_rps * 60.0)?;

        match state {
            State::SwitchToSpindle => {
                argon.set_control_mode(ControlMode::Velocity)?;

                state = State::Spindle;
            }
            State::Spindle => {
                if orient_enable {
                    state = State::SwitchToOrient;

                    continue;
                }

                log::debug!(
                    "Change setpoint from {} to {}",
                    current_velocity_setpoint_rps,
                    new_velocity_rps,
                );

                argon.set_velocity_rps(new_velocity_rps)?;
            }
            State::SwitchToOrient => {
                // let current_velocity = argon.velocity()?;

                // log::trace!("Spindle velocity {}", current_velocity_rps);

                // Wait for velocity to reach zero before switching to orient mode.
                if current_velocity_rps == 0.0 {
                    log::trace!("Orient counts {:?}", orient_position_counts);

                    argon.home(orient_position_counts as i32)?;

                    state = State::Orienting
                }
            }
            State::Orienting => {
                // Set status and change mode when orient completes
                if !argon.status()?.homing {
                    pins.is_oriented.set_value(true)?;

                    state = State::Oriented
                }
            }
            State::Oriented => {
                if orient_enable {
                    pins.is_oriented.set_value(false)?;

                    state = State::SwitchToOrient;
                } else {
                    pins.is_oriented.set_value(false)?;

                    state = State::SwitchToSpindle;
                }
            }
            _ => (),
        }

        thread::sleep(Duration::from_millis(50));
    }

    // // Main control loop
    // while !comp.should_exit() {
    //     let new_mode = if *pins.orient_enable.value()? {
    //         ControlMode::Position
    //     } else {
    //         ControlMode::Velocity
    //     };

    //     if new_mode != current_mode {
    //         log::debug!("Changing control mode to {}", new_mode);

    //         argon.set_control_mode(new_mode)?;
    //         current_mode = new_mode;
    //     }

    //     match current_mode {
    //         ControlMode::Velocity => {
    //             // SAFETEY: Default to 0 RPM if some error occurred.
    //             let new_speed = *pins.spindle_speed.value().unwrap_or_else(|e| {
    //                 log::error!(
    //                     "Failed to get spindle speed value: {}. Defaulting to 0.0",
    //                     e
    //                 );

    //                 &0.0
    //             });

    //             if new_speed != current_speed {
    //                 current_speed = new_speed;

    //                 log::info!("Changing setpoint to {} RPM", new_speed);

    //                 argon.set_absolute_setpoint(new_speed as i32)?;
    //             }
    //         }
    //         ControlMode::Position => {
    //             // SAFETEY: Default to current pos (no movement) if error occurred
    //             let new_pos = *pins.orient_angle.value().unwrap_or_else(|e| {
    //                 log::error!(
    //                     "Failed to get spindle speed value: {}. Defaulting to 0.0",
    //                     e
    //                 );

    //                 &0.0
    //             });

    //             log::debug!("{:?}", argon.status()?.homing);

    //             if new_pos != current_pos {
    //                 current_pos = new_pos;

    //                 // log::info!("Changing setpoint to {}", new_pos);

    //                 // argon.set_absolute_setpoint(new_pos as i32)?;
    //                 log::debug!("Homing");
    //                 argon.home(0)?;
    //             }
    //         }
    //         mode => panic!("Unsupported control mode {}", mode),
    //     }

    //     thread::sleep(Duration::from_millis(50));
    // }

    // Bare minimum safe state on shutdown.
    // FIXME: Check if I need to set anything else.
    argon.set_velocity_rps(0.0)?;

    Ok(())
}
