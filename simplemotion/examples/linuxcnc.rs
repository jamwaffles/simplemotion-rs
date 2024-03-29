// NOTE: Uncomment the `// ONLY required for non-realtime builds` section in
// `linuxcnc-hal-rs/linuxcnc-hal-sys/build.rs` to make this example link properly.

use linuxcnc_hal::{
    error::PinRegisterError, hal_pin::InputPin, hal_pin::OutputPin, prelude::*, HalComponent,
    RegisterResources, Resources,
};
use simplemotion::{Argon, ControlMode};
use smol::{LocalExecutor, Timer};
use std::{error::Error, time::Duration};

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

    /// Whether the drive is in an error state or not.
    drive_error: OutputPin<bool>,
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
            drive_error: comp.register_pin("drive-error")?,
        })
    }
}

/// Update interval delay in ms
const DEFAULT_UPDATE_INTERVAL: u64 = 10;

#[derive(Debug, Copy, Clone)]
enum State {
    Idle,
    SwitchToSpindle,
    Spindle,
    SwitchToOrient,
    Orienting,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _level = std::env::var("ARGON_LOG_LEVEL")
        .unwrap_or("info".to_string())
        .parse()
        .unwrap_or(log::LevelFilter::Info);

    let _ = rtapi_logger::init();

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

    let update_interval = std::env::args()
        .nth(3)
        .and_then(|interval| interval.parse().ok())
        .unwrap_or(DEFAULT_UPDATE_INTERVAL);

    let update_interval = Duration::from_millis(update_interval);

    log::info!("Starting Argon driver using device {device}, drive address {address}, update interval {} ms", update_interval.as_millis());

    let mut argon = Argon::connect(&device, address)?;

    argon.clear_faults()?;

    let comp: HalComponent<Comp> = HalComponent::new("argon")?;
    let pins = comp.resources();

    log::trace!("Pins: {:?}", pins);

    // Initial state on startup.
    let mut state = State::Idle;

    let _local_ex = LocalExecutor::new();

    let _timer = Timer::interval(update_interval);

    let mut error = false;

    // future::block_on(local_ex.run(async {
    // Main control loop
    while !comp.should_exit() {
        if error {
            error = argon.reconnect().is_err();
        }

        match loop_tick(&mut argon, &pins, state) {
            Ok(new_state) => state = new_state,
            Err(e) => {
                log::error!("Argon driver error: {}, attempting to reconnect", e);

                error = true;
            }
        }

        // timer.next().await;
        std::thread::sleep(update_interval);
    }

    //     Ok::<(), Box<dyn Error>>(())
    // }))?;

    // Bare minimum safe state on shutdown.
    // FIXME: Check if I need to set anything else.
    argon.set_velocity_rps(0.0)?;

    Ok(())
}

fn loop_tick(argon: &mut Argon, pins: &Comp, mut state: State) -> Result<State, Box<dyn Error>> {
    let current_velocity_setpoint_rps = argon.setpoint_rps()?;
    let new_velocity_rps = *pins.spindle_speed_rps.value()?;
    let current_velocity_rps = argon.velocity_rps()?;

    let orient_enable = *pins.orient_enable.value()?;

    pins.spindle_fb_rps.set_value(current_velocity_rps)?;
    pins.spindle_fb_rpm.set_value(current_velocity_rps * 60.0)?;

    pins.drive_error.set_value(argon.faults()?.any())?;

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

            argon.set_velocity_rps(0.0)?;

            if argon.faults()?.any() {
                log::debug!(
                    "Drive has faults: {:?}, attempting to reset",
                    argon.faults()?
                );

                argon.clear_faults()?;
            }

            // If we couldn't clear faults, transition to idle. We can check pins.drive_error to
            // report fault status.
            if argon.faults()?.any() {
                log::error!("Could not clear faults");

                state = State::Idle;
            } else {
                state = State::Spindle;
            }
        }
        State::Spindle => {
            if orient_enable {
                log::debug!("Switching to orient");

                state = State::SwitchToOrient;
            } else if (new_velocity_rps - current_velocity_setpoint_rps).abs() > 0.01 {
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
                argon.set_homing_complete()?;

                pins.is_oriented.set_value(true)?;

                state = State::Idle
            }
        }
    }

    Ok(state)
}
