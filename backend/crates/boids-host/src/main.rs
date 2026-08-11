use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::Duration;

use boids_core::bounds::{BoundaryMode, Bounds};
use boids_core::math::Vec3;
use boids_core::params::SimulationParams;
use boids_core::spawn::SpawnConfig;
use boids_core::world::{World, WorldSettings};
use serde_json::{Value, json};

const FIXED_DT: f32 = 1.0 / 60.0;

fn snapshot_json(world: &World, tick: u64) -> Value {
    let boids = world
        .boids()
        .iter()
        .map(|boid| {
            json!({
                "id": boid.id,
                "position": {
                    "x": boid.position.x,
                    "y": boid.position.y,
                    "z": boid.position.z,
                },
                "velocity": {
                    "x": boid.velocity.x,
                    "y": boid.velocity.y,
                    "z": boid.velocity.z,
                },
            })
        })
        .collect::<Vec<_>>();

    json!({
        "tick": tick,
        "boids": boids,
    })
}

fn write_snapshot(writer: &mut impl Write, world: &World, tick: u64) -> io::Result<()> {
    let json = serde_json::to_string(&snapshot_json(world, tick)).map_err(io::Error::other)?;

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

    let spawn_config = SpawnConfig::new(500, 123, 100.0);
    let mut world = World::from_config(spawn_config, world_settings);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let step_duration = Duration::from_secs_f32(FIXED_DT);
    let mut tick = 0;

    write_snapshot(&mut writer, &world, tick)?;

    loop {
        thread::sleep(step_duration);

        world.step(FIXED_DT);
        tick += 1;

        if let Err(error) = write_snapshot(&mut writer, &world, tick) {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }

            return Err(error);
        }
    }
}
