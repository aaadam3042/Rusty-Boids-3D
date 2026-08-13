use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};

use boids_core::bounds::{BoundaryMode, Bounds};
use boids_core::math::Vec3;
use boids_core::params::SimulationParams;
use boids_core::spawn::SpawnConfig;
use boids_core::world::{World, WorldSettings};
use serde::Serialize;

const FIXED_DT: f32 = 1.0 / 60.0;
const HEALTH_WINDOW: Duration = Duration::from_secs(1);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorldSnapshot {
    tick: u64,
    boids: Vec<BoidSnapshot>,
    health: HostHealth,
}

#[derive(Debug, Serialize)]
struct BoidSnapshot {
    id: u32,
    position: Vec3Snapshot,
    velocity: Vec3Snapshot,
}

#[derive(Debug, Serialize)]
struct Vec3Snapshot {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for Vec3Snapshot {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostHealth {
    simulation_time_seconds: f64,
    fixed_dt_seconds: f64,
    real_time_factor: f64,
    real_time_factor_ready: bool,
    deadline_lateness_ms: f64,
    last_step_ms: f64,
    previous_publish_ms: f64,
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
    ) -> HostHealth {
        let window_duration = now.saturating_duration_since(self.window_started_at);

        if window_duration >= HEALTH_WINDOW {
            let ticks_advanced = tick.saturating_sub(self.window_started_tick);
            let simulated_seconds = ticks_advanced as f64 * FIXED_DT as f64;

            self.real_time_factor = simulated_seconds / window_duration.as_secs_f64();
            self.real_time_factor_ready = true;
            self.window_started_at = now;
            self.window_started_tick = tick;
        }

        HostHealth {
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

fn make_snapshot(world: &World, tick: u64, health: HostHealth) -> WorldSnapshot {
    let boids = world
        .boids()
        .iter()
        .map(|boid| BoidSnapshot {
            id: boid.id,
            position: boid.position.into(),
            velocity: boid.velocity.into(),
        })
        .collect();

    WorldSnapshot {
        tick,
        boids,
        health,
    }
}

fn write_snapshot(writer: &mut impl Write, snapshot: &WorldSnapshot) -> io::Result<()> {
    let json = serde_json::to_string(snapshot).map_err(io::Error::other)?;

    writeln!(writer, "{json}")?;
    writer.flush()
}

fn main() -> io::Result<()> {
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

    let spawn_config = SpawnConfig::new(1000, 123, 100.0);
    let mut world = World::from_config(spawn_config, world_settings);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let tick_duration = Duration::from_secs_f32(FIXED_DT);
    let mut tick = 0;
    let started_at = Instant::now();
    let mut next_tick = started_at + tick_duration;
    let mut health_tracker = HostHealthTracker::new(started_at, tick);

    let initial_health = health_tracker.sample(tick, started_at, started_at, Duration::ZERO);
    let publish_started_at = Instant::now();
    let initial_snapshot = make_snapshot(&world, tick, initial_health);
    write_snapshot(&mut writer, &initial_snapshot)?;
    health_tracker.record_publish(publish_started_at.elapsed());

    loop {
        let step_started_at = Instant::now();
        world.step(FIXED_DT);
        let last_step_duration = step_started_at.elapsed();
        tick += 1;

        thread::sleep(next_tick.saturating_duration_since(Instant::now()));

        let health = health_tracker.sample(tick, Instant::now(), next_tick, last_step_duration);
        let publish_started_at = Instant::now();
        let snapshot = make_snapshot(&world, tick, health);

        if let Err(error) = write_snapshot(&mut writer, &snapshot) {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }

            return Err(error);
        }

        health_tracker.record_publish(publish_started_at.elapsed());
        next_tick += tick_duration;
    }
}
