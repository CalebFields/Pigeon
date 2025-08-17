# Pigeon Coding Standards (v0)

This document captures the initial coding standards for the Pigeon Rust project.

## Language & Tooling
- Rust (Edition 2021)
- Build/test via `cargo`
- Formatting: `rustfmt` (configured via `rustfmt.toml` if needed)
- Linting: `clippy` (treat warnings as errors in CI)
- Security/licensing: optional `cargo-deny`
- Docs: `cargo doc` with examples

## General Guidelines
- Prefer clarity over cleverness; optimize for readability
- Use descriptive names; avoid abbreviations and 1–2 character identifiers
- Guard clauses over deep nesting
- Handle errors early; return early on invalid state
- Avoid global mutable state; prefer explicit dependency injection
- Keep modules small and cohesive; split by domain (`messaging`, `network`, `storage`, `ui`)

## Rust Specific
- Use `Result<T, E>` for fallible APIs; bubble errors with `?`
- Error types:
  - Library crates: define domain errors via `thiserror`
  - Binaries/CLI: use `anyhow` at boundaries for ergonomics
- Concurrency: prefer async (`tokio`) primitives; avoid blocking in async contexts
- Do not `unwrap()` or `expect()` outside tests/examples; prefer meaningful error propagation
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` where appropriate; avoid unnecessary clones
- Prefer `&str` and slices for APIs over owned `String`/`Vec` when possible
- Document public items with rustdoc comments; provide examples that compile (`rustdoc`-tested)

## Modules & Layout
- One responsibility per module; re-export minimal public surface in `mod.rs`
- Keep `src/lib.rs` as the crate façade; keep `src/main.rs` thin (argument parsing + invocation)
- Feature flags guarded behind `#[cfg(feature = "...")]`

## Error Handling
- Map external errors into domain errors at boundaries
- Log context on failure using `tracing`/`log`; avoid logging sensitive data (keys, plaintext)
- For CLI, produce actionable messages; include `--verbose` for extra detail

## Security
- Never log secrets, private keys, raw plaintext, or unredacted message payloads
- Zeroize key material where feasible; restrict file permissions when persisting secrets
- Validate all inputs at boundaries (network frames, config files)
- Use constant-time comparisons for authentication-relevant checks

## Testing
- Unit tests co-located with modules (`#[cfg(test)]`)
- Integration tests under `tests/`
- Deterministic tests: avoid wall-clock time and nondeterministic sources
- Property tests optional for parsers/encoders

## Formatting & Style
- Enforced by `rustfmt`
- Run `cargo clippy --all-targets --all-features -D warnings` locally before commits
- Keep functions small; extract helpers for complex logic

## Commit Discipline
- Conventional commits recommended (`feat:`, `fix:`, `docs:`, etc.)
- Small, focused PRs with clear rationale


