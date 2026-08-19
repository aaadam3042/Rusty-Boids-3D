# boids-host protocol

`boids-host` and Unity communicate over newline-delimited JSON (NDJSON). Unity writes commands to
the child process's stdin. The host writes protocol messages to stdout and diagnostics to stderr.
Each line is one complete JSON object. Protocol version `1` is the only supported version.

## Connection sequence

Unity attaches its stdout and stderr readers, starts asynchronous reading, and sends:

```json
{"type":"hello","protocolVersion":1}
```

The host does not publish snapshots before accepting `hello`. It responds with the effective
Rust-owned startup state:

```json
{"type":"ready","protocolVersion":1,"bounds":{"min":{"x":0.0,"y":0.0,"z":0.0},"max":{"x":100.0,"y":100.0,"z":100.0}},"weights":{"cohesion":3.0,"alignment":1.0,"separation":180.0}}
```

After `ready`, the host starts publishing snapshots. Sending another compatible `hello` is safe and
causes the host to resend the current `ready` state.

## Unity-to-host commands

Replace all three flocking weights atomically:

```json
{"type":"setWeights","weights":{"cohesion":4.0,"alignment":1.5,"separation":120.0}}
```

Shut the host down cleanly:

```json
{"type":"shutdown"}
```

Closing stdin also shuts the host down.

## Host-to-Unity messages

A world snapshot has the following shape:

```json
{"type":"snapshot","tick":42,"boids":[{"id":0,"position":{"x":1.0,"y":2.0,"z":3.0},"velocity":{"x":4.0,"y":5.0,"z":6.0}}],"health":{"simulationTimeSeconds":0.7,"fixedDtSeconds":0.016666668,"realTimeFactor":1.0,"realTimeFactorReady":true,"deadlineLatenessMs":0.1,"lastStepMs":1.2,"previousPublishMs":0.4}}
```

A successful weight update returns the effective values:

```json
{"type":"weightsUpdated","weights":{"cohesion":4.0,"alignment":1.5,"separation":120.0}}
```

Rejected commands return an error without stopping the simulation:

```json
{"type":"error","code":"invalidWeights","message":"invalid weights: MustBeNonNegative(\"alignment_weight\")"}
```

Error codes in version `1` are:

- `malformedCommand`
- `unsupportedProtocolVersion`
- `invalidWeights`
- `notReady`

## Delivery and ownership rules

- The host waits for `hello`, sends `ready`, and only then sends snapshots.
- stdout contains protocol lines only; stderr contains diagnostics only.
- Unity processes every control message in FIFO order but may retain only the newest unread snapshot.
- Bounds, weights, boid state, timing, and validation remain authoritative in Rust.
- Unity uses bounds for presentation. Unity colliders do not constrain Rust-owned boid positions.
