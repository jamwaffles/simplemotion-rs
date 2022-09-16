// NOTE: Uncomment the `// ONLY required for non-realtime builds` section in
// `linuxcnc-hal-rs/linuxcnc-hal-sys/build.rs` to make this example link properly.

use linuxcnc_hal::{
    error::PinRegisterError, hal_pin::InputPin, hal_pin::OutputPin, prelude::*, HalComponent,
    RegisterResources, Resources,
};
use simplemotion::{Argon, ControlMode};
use std::{error::Error, thread, time::Duration};

#[derive(Debug)]
struct Comp {
    /// Whether to start orienting the spindle or not.
    orient_enable: InputPin<bool>,

    /// Orient position relative to index pulse in degrees.
    orient_angle: InputPin<f64>,

    /// Spindle speed setpoint in Revolutions Per Second.
    spindle_speed_rps: InputPin<f64>,

    /// Flag to signal that the orient is complete.
    is_oriented: OutputPin<bool>,

    /// Current measured motor RPS.
    spindle_fb_rps: OutputPin<f64>,

    /// Current measured motor RPM.
    spindle_fb_rpm: OutputPin<f64>,
}

impl Resources for Comp {
    type RegisterError = PinRegisterError;

    fn register_resources(comp: &RegisterResources) -> Result<Self, Self::RegisterError> {
        Ok(Comp {
            orient_enable: comp.register_pin("orient-enable")?,
            orient_angle: comp.register_pin("orient-angle")?,
            spindle_speed_rps: comp.register_pin("spindle-speed-rps")?,
            is_oriented: comp.register_pin("is-oriented")?,
            spindle_fb_rps: comp.register_pin("spindle-fb-rps")?,
            spindle_fb_rpm: comp.register_pin("spindle-fb-rpm")?,
        })
    }
}

/// Update interval delay in ms
const UPDATE_INTERVAL: u64 = 10;

#[derive(Debug, Copy, Clone)]
enum State {
    Idle,
    SwitchToSpindle,
    Spindle,
    SwitchToOrient,
    Orienting,
}

fn main() -> Result<(), Box<dyn Error>> {
    let level = std::env::var("ARGON_LOG_LEVEL")
        .unwrap_or("info".to_string())
        .parse()
        .unwrap_or(log::LevelFilter::Info);

    rtapi_logger::init(level);

    match inner() {
        Ok(res) => Ok(res),
        Err(e) => {
            log::error!("{e}");

            Err(e)
        }
    }
}

fn inner() -> Result<(), Box<dyn Error>> {
    let device = std::env::args().nth(1).expect("Device name/path required");
    let address: u8 = std::env::args()
        .nth(2)
        .expect("Device address is required")
        .parse()
        .expect("Device address must be a number from 1 - 255");

    log::info!("Starting Argon driver using device {device}, drive address {address}");

    let mut argon = Argon::connect(&device, address)?;

    argon.clear_faults()?;

    let comp: HalComponent<Comp> = HalComponent::new("argon")?;
    let pins = comp.resources();

    log::trace!("Pins: {:?}", pins);

    // Initial state on startup.
    let mut state = State::Idle;

    // Main control loop
    while !comp.should_exit() {
        let current_velocity_setpoint_rps = argon.setpoint_rps()?;
        let new_velocity_rps = *pins.spindle_speed_rps.value()?;
        let current_velocity_rps = argon.velocity_rps()?;

        let orient_enable = *pins.orient_enable.value()?;

        pins.spindle_fb_rps.set_value(current_velocity_rps)?;
        pins.spindle_fb_rpm.set_value(current_velocity_rps * 60.0)?;

        match state {
            State::Idle => {
                if orient_enable {
                    log::debug!("Beginning orient...");

                    pins.is_oriented.set_value(false)?;

                    state = State::SwitchToOrient;
                } else if new_velocity_rps != 0.0 {
                    log::debug!("Switching to velocity mode...");

                    pins.is_oriented.set_value(false)?;

                    state = State::SwitchToSpindle;
                }
            }
            State::SwitchToSpindle => {
                log::trace!("Switching to spindle mode...");

                argon.set_control_mode(ControlMode::Velocity)?;

                state = State::Spindle;
            }
            State::Spindle => {
                if orient_enable {
                    log::debug!("Switching to orient");

                    state = State::SwitchToOrient;

                    continue;
                }

                if (new_velocity_rps - current_velocity_setpoint_rps).abs() > 0.0001 {
                    log::debug!(
                        "Change setpoint from {} to {}",
                        current_velocity_setpoint_rps,
                        new_velocity_rps,
                    );

                    argon.set_velocity_rps(new_velocity_rps)?;
                }
            }
            State::SwitchToOrient => {
                log::trace!(
                    "Switching to orient... spindle vel: {}",
                    current_velocity_rps
                );

                argon.set_velocity_rps(0.0)?;

                // Wait for velocity to reach zero before switching to orient mode.
                if current_velocity_rps == 0.0 {
                    log::debug!("Orient angle (degrees): {:?}", *pins.orient_angle.value()?);

                    argon.home(*pins.orient_angle.value()?)?;

                    state = State::Orienting
                }
            }
            State::Orienting => {
                log::trace!("Orienting...");

                // Set status and change mode when orient completes
                if !argon.status()?.homing {
                    log::debug!("Oriented");

                    // Reset homing flag so we can orient multiple times in a row
                    argon.set_homing_complete();

                    pins.is_oriented.set_value(true)?;

                    state = State::Idle
                }
            }
        }

        thread::sleep(Duration::from_millis(UPDATE_INTERVAL));
    }

    // Bare minimum safe state on shutdown.
    // FIXME: Check if I need to set anything else.
    argon.set_velocity_rps(0.0)?;

    Ok(())
}
