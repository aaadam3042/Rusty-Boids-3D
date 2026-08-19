mod protocol;

use std::io::{self, BufRead, BufWriter, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use boids_core::bounds::{BoundaryMode, Bounds};
use boids_core::math::Vec3;
use boids_core::params::SimulationParams;
use boids_core::spawn::SpawnConfig;
use boids_core::world::{World, WorldSettings};
use protocol::{
    ClientCommand, HostHealthSnapshot, HostMessage, PROTOCOL_VERSION, ProtocolErrorCode,
    WeightsSnapshot,
};

const FIXED_DT: f32 = 1.0 / 60.0;
const HEALTH_WINDOW: Duration = Duration::from_secs(1);

enum InputEvent {
    Command(ClientCommand),
    Malformed(String),
    Closed,
}

struct HostHealthTracker {
    window_started_at: Instant,
    window_started_tick: u64,
    real_time_factor: f64,
    real_time_factor_ready: bool,
    previous_publish_duration: Duration,
}

impl HostHealthTracker {
    fn new(now: Instant, tick: u64) -> Self {
        Self {
            window_started_at: now,
            window_started_tick: tick,
            real_time_factor: 0.0,
            real_time_factor_ready: false,
            previous_publish_duration: Duration::ZERO,
        }
    }

    fn sample(
        &mut self,
        tick: u64,
        now: Instant,
        deadline: Instant,
        last_step_duration: Duration,
    ) -> HostHealthSnapshot {
        let window_duration = now.saturating_duration_since(self.window_started_at);

        if window_duration >= HEALTH_WINDOW {
            let ticks_advanced = tick.saturating_sub(self.window_started_tick);
            let simulated_seconds = ticks_advanced as f64 * FIXED_DT as f64;

            self.real_time_factor = simulated_seconds / window_duration.as_secs_f64();
            self.real_time_factor_ready = true;
            self.window_started_at = now;
            self.window_started_tick = tick;
        }

        HostHealthSnapshot {
            simulation_time_seconds: tick as f64 * FIXED_DT as f64,
            fixed_dt_seconds: FIXED_DT as f64,
            real_time_factor: self.real_time_factor,
            real_time_factor_ready: self.real_time_factor_ready,
            deadline_lateness_ms: milliseconds(now.saturating_duration_since(deadline)),
            last_step_ms: milliseconds(last_step_duration),
            previous_publish_ms: milliseconds(self.previous_publish_duration),
        }
    }

    fn record_publish(&mut self, duration: Duration) {
        self.previous_publish_duration = duration;
    }
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn spawn_input_reader() -> Receiver<InputEvent> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let stdin = io::stdin();

        for line_result in stdin.lock().lines() {
            let event = match line_result {
                Ok(line) => match serde_json::from_str::<ClientCommand>(&line) {
                    Ok(command) => InputEvent::Command(command),
                    Err(error) => InputEvent::Malformed(error.to_string()),
                },
                Err(error) => {
                    eprintln!("failed to read boids-host stdin: {error}");
                    break;
                }
            };

            if sender.send(event).is_err() {
                return;
            }
        }

        let _ = sender.send(InputEvent::Closed);
    });

    receiver
}

fn write_message(writer: &mut impl Write, message: &HostMessage) -> io::Result<()> {
    let json = serde_json::to_string(message).map_err(io::Error::other)?;
    writeln!(writer, "{json}")?;
    writer.flush()
}

fn write_error(
    writer: &mut impl Write,
    code: ProtocolErrorCode,
    message: impl Into<String>,
) -> io::Result<()> {
    write_message(writer, &HostMessage::error(code, message))
}

fn protocol_version_is_supported(
    protocol_version: u32,
    writer: &mut impl Write,
) -> io::Result<bool> {
    if protocol_version == PROTOCOL_VERSION {
        return Ok(true);
    }

    write_error(
        writer,
        ProtocolErrorCode::UnsupportedProtocolVersion,
        format!("unsupported protocol version {protocol_version}; expected {PROTOCOL_VERSION}"),
    )?;
    Ok(false)
}

fn wait_for_hello(
    receiver: &Receiver<InputEvent>,
    writer: &mut impl Write,
    world: &World,
) -> io::Result<bool> {
    loop {
        match receiver.recv() {
            Ok(InputEvent::Command(ClientCommand::Hello { protocol_version })) => {
                if protocol_version_is_supported(protocol_version, writer)? {
                    write_message(writer, &HostMessage::ready(world))?;
                    return Ok(true);
                }
            }
            Ok(InputEvent::Command(ClientCommand::SetWeights { .. })) => {
                write_error(
                    writer,
                    ProtocolErrorCode::NotReady,
                    "hello must complete before weights can be changed",
                )?;
            }
            Ok(InputEvent::Command(ClientCommand::Shutdown)) | Ok(InputEvent::Closed) | Err(_) => {
                return Ok(false);
            }
            Ok(InputEvent::Malformed(message)) => {
                write_error(writer, ProtocolErrorCode::MalformedCommand, message)?;
            }
        }
    }
}

fn apply_weights(
    world: &mut World,
    weights: WeightsSnapshot,
    writer: &mut impl Write,
) -> io::Result<()> {
    match world.set_weights(weights.cohesion, weights.alignment, weights.separation) {
        Ok(()) => write_message(writer, &HostMessage::weights_updated(world.params())),
        Err(error) => write_error(
            writer,
            ProtocolErrorCode::InvalidWeights,
            format!("invalid weights: {error:?}"),
        ),
    }
}

fn process_pending_input(
    receiver: &Receiver<InputEvent>,
    writer: &mut impl Write,
    world: &mut World,
) -> io::Result<bool> {
    loop {
        match receiver.try_recv() {
            Ok(InputEvent::Command(ClientCommand::Hello { protocol_version })) => {
                if protocol_version_is_supported(protocol_version, writer)? {
                    write_message(writer, &HostMessage::ready(world))?;
                }
            }
            Ok(InputEvent::Command(ClientCommand::SetWeights { weights })) => {
                apply_weights(world, weights, writer)?;
            }
            Ok(InputEvent::Command(ClientCommand::Shutdown)) | Ok(InputEvent::Closed) => {
                return Ok(false);
            }
            Ok(InputEvent::Malformed(message)) => {
                write_error(writer, ProtocolErrorCode::MalformedCommand, message)?;
            }
            Err(TryRecvError::Empty) => return Ok(true),
            Err(TryRecvError::Disconnected) => return Ok(false),
        }
    }
}

fn run() -> io::Result<()> {
    let simulation_params = SimulationParams::default();

    let min_bound = Vec3::new(0.0, 0.0, 0.0);
    let max_bound = Vec3::new(100.0, 100.0, 100.0);
    let bounds =
        Bounds::try_new(min_bound, max_bound).expect("hard-coded world bounds should be valid");

    let world_settings = WorldSettings {
        params: simulation_params,
        bounds,
        boundary_mode: BoundaryMode::Bounce,
        // boundary_mode: BoundaryMode::SoftTurn { margin: 10.0, turn_acceleration: 500.0 },
    };

    let spawn_config = SpawnConfig::new(2000, 123, 100.0);
    let mut world = World::from_config(spawn_config, world_settings);

    let input = spawn_input_reader();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    if !wait_for_hello(&input, &mut writer, &world)? {
        return Ok(());
    }

    let tick_duration = Duration::from_secs_f32(FIXED_DT);
    let mut tick = 0;
    let started_at = Instant::now();
    let mut next_tick = started_at + tick_duration;
    let mut health_tracker = HostHealthTracker::new(started_at, tick);

    let initial_health = health_tracker.sample(tick, started_at, started_at, Duration::ZERO);
    let publish_started_at = Instant::now();
    let initial_snapshot = HostMessage::snapshot(&world, tick, initial_health);
    write_message(&mut writer, &initial_snapshot)?;
    health_tracker.record_publish(publish_started_at.elapsed());

    loop {
        if !process_pending_input(&input, &mut writer, &mut world)? {
            return Ok(());
        }

        let step_started_at = Instant::now();
        world.step(FIXED_DT);
        let last_step_duration = step_started_at.elapsed();
        tick += 1;

        thread::sleep(next_tick.saturating_duration_since(Instant::now()));

        let health = health_tracker.sample(tick, Instant::now(), next_tick, last_step_duration);
        let publish_started_at = Instant::now();
        let snapshot = HostMessage::snapshot(&world, tick, health);
        write_message(&mut writer, &snapshot)?;

        health_tracker.record_publish(publish_started_at.elapsed());
        next_tick += tick_duration;
    }
}

fn main() -> io::Result<()> {
    match run() {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}
