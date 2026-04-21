# agent-client-rs

Rust HTTP client for the Listo agent REST API. Published as `listo-agent-client`
on crates.io.

## Usage

```rust
use listo_agent_client::AgentClient;

let client = AgentClient::new("http://localhost:8080", token);
let nodes = client.nodes().list().await?;
```

## Build

```bash
cargo build
cargo test
```

## Dependencies

- [`contracts`](../contracts) — wire types (`listo-spi`)

## Sibling clients

Keep endpoint shape, request/response types, and error handling consistent across:

- [`agent-client-ts`](../agent-client-ts) — TypeScript
- [`agent-client-dart`](../agent-client-dart) — Dart/Flutter

Part of the [listo-ai workspace](../).
