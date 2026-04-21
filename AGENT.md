# AGENT.md — agent-client-rs

Rust HTTP client for the Listo AI agent REST API (`listo-agent-client` on crates.io).

---

## Skills

See [SKILLS/rust.md](../SKILLS/rust.md) for the full skill map.

Quick reference for this repo:

| Task | Skill path |
|------|------------|
| Any Rust code | `~/.agents/skills/rust-skills/SKILL.md` |
| HTTP client / API design | `~/.agents/skills/api-and-interface-design/SKILL.md` |
| TDD | `~/.agents/skills/test-driven-development/SKILL.md` |
| Debugging | `~/.agents/skills/debugging-and-error-recovery/SKILL.md` |
| Security | `~/.agents/skills/security-and-hardening/SKILL.md` |
| Code review | `~/.agents/skills/code-review-and-quality/SKILL.md` |

---

## Tech Stack

- **Language**: Rust (≥ 1.88)
- **Package manager**: `cargo`
- **Registry**: crates.io (`listo-agent-client`)
- **HTTP**: `reqwest` (check `Cargo.toml` for current deps before adding alternatives)
- **Serialization**: `serde` / `serde_json`

## Reference implementations

| Repo | Language | Path |
|------|----------|------|
| `agent-client-ts` | TypeScript | `../agent-client-ts/` |
| `agent-client-dart` | Dart | `../agent-client-dart/` |

Check both when implementing a new API method — endpoint shape, request/response types, and error handling must be consistent across all clients.

## Workspace commands

```bash
cargo build                 # build
cargo test                  # run tests
cargo clippy -- -D warnings # lint
cargo fmt --check           # format check
cargo publish --dry-run     # check publishability
```

## Conventions

- `#![forbid(unsafe_code)]` — no unsafe.
- No `.unwrap()` / `.expect()` in library code. Use `Result` + `thiserror`.
- All public types and functions must have doc comments.
- Error types in `error.rs` — mirror shapes from TS client where possible.
- No `dynamic`-equivalent: avoid `Box<dyn Any>` on public API surface.
