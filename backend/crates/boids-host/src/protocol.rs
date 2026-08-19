use boids_core::bounds::Bounds;
use boids_core::math::Vec3;
use boids_core::params::SimulationParams;
use boids_core::world::World;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ClientCommand {
    #[serde(rename = "hello")]
    Hello {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
    },
    #[serde(rename = "setWeights")]
    SetWeights { weights: WeightsSnapshot },
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum HostMessage {
    #[serde(rename = "ready")]
    Ready {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        bounds: BoundsSnapshot,
        weights: WeightsSnapshot,
    },
    #[serde(rename = "snapshot")]
    Snapshot {
        tick: u64,
        boids: Vec<BoidSnapshot>,
        health: HostHealthSnapshot,
    },
    #[serde(rename = "weightsUpdated")]
    WeightsUpdated { weights: WeightsSnapshot },
    #[serde(rename = "error")]
    Error {
        code: ProtocolErrorCode,
        message: String,
    },
}

impl HostMessage {
    pub fn ready(world: &World) -> Self {
        Self::Ready {
            protocol_version: PROTOCOL_VERSION,
            bounds: world.bounds().into(),
            weights: world.params().into(),
        }
    }

    pub fn snapshot(world: &World, tick: u64, health: HostHealthSnapshot) -> Self {
        let boids = world
            .boids()
            .iter()
            .map(|boid| BoidSnapshot {
                id: boid.id,
                position: boid.position.into(),
                velocity: boid.velocity.into(),
            })
            .collect();

        Self::Snapshot {
            tick,
            boids,
            health,
        }
    }

    pub fn weights_updated(params: &SimulationParams) -> Self {
        Self::WeightsUpdated {
            weights: params.into(),
        }
    }

    pub fn error(code: ProtocolErrorCode, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WeightsSnapshot {
    pub cohesion: f32,
    pub alignment: f32,
    pub separation: f32,
}

impl From<&SimulationParams> for WeightsSnapshot {
    fn from(params: &SimulationParams) -> Self {
        Self {
            cohesion: params.cohesion_weight(),
            alignment: params.alignment_weight(),
            separation: params.separation_weight(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BoundsSnapshot {
    min: Vec3Snapshot,
    max: Vec3Snapshot,
}

impl From<&Bounds> for BoundsSnapshot {
    fn from(bounds: &Bounds) -> Self {
        Self {
            min: bounds.min().into(),
            max: bounds.max().into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BoidSnapshot {
    id: u32,
    position: Vec3Snapshot,
    velocity: Vec3Snapshot,
}

#[derive(Debug, Serialize)]
pub struct Vec3Snapshot {
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
pub struct HostHealthSnapshot {
    pub simulation_time_seconds: f64,
    pub fixed_dt_seconds: f64,
    pub real_time_factor: f64,
    pub real_time_factor_ready: bool,
    pub deadline_lateness_ms: f64,
    pub last_step_ms: f64,
    pub previous_publish_ms: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolErrorCode {
    MalformedCommand,
    UnsupportedProtocolVersion,
    InvalidWeights,
    NotReady,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialises_hello_command() {
        let command =
            serde_json::from_str::<ClientCommand>(r#"{"type":"hello","protocolVersion":1}"#)
                .expect("expected valid hello command");

        assert_eq!(
            command,
            ClientCommand::Hello {
                protocol_version: 1
            }
        );
    }

    #[test]
    fn deserialises_set_weights_command() {
        let command = serde_json::from_str::<ClientCommand>(
            r#"{"type":"setWeights","weights":{"cohesion":4.0,"alignment":5.0,"separation":6.0}}"#,
        )
        .expect("expected valid setWeights command");

        assert_eq!(
            command,
            ClientCommand::SetWeights {
                weights: WeightsSnapshot {
                    cohesion: 4.0,
                    alignment: 5.0,
                    separation: 6.0,
                }
            }
        );
    }

    #[test]
    fn serialises_ready_with_discriminator_and_camel_case_version() {
        let message = HostMessage::Ready {
            protocol_version: PROTOCOL_VERSION,
            bounds: BoundsSnapshot {
                min: Vec3::ZERO.into(),
                max: Vec3::new(100.0, 100.0, 100.0).into(),
            },
            weights: WeightsSnapshot {
                cohesion: 3.0,
                alignment: 1.0,
                separation: 180.0,
            },
        };

        let value = serde_json::to_value(message).expect("expected serialisable ready message");

        assert_eq!(value["type"], json!("ready"));
        assert_eq!(value["protocolVersion"], json!(1));
        assert_eq!(value["bounds"]["max"]["z"], json!(100.0));
        assert_eq!(value["weights"]["separation"], json!(180.0));
    }

    #[test]
    fn serialises_protocol_error_code_as_camel_case() {
        let message = HostMessage::error(
            ProtocolErrorCode::UnsupportedProtocolVersion,
            "unsupported version",
        );

        let value = serde_json::to_value(message).expect("expected serialisable error message");

        assert_eq!(value["type"], json!("error"));
        assert_eq!(value["code"], json!("unsupportedProtocolVersion"));
    }
}
