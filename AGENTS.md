# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` wires the winit event loop, camera, GPU drawer, and server.
- Rendering lives under `src/gpu/` (camera controllers, shaders, textures like `*.png`, mesh/buffer helpers).
- World logic sits in `src/server/` (voxel types, chunks, world generation).
- Input and console systems are `src/input/` and `src/console/`; threading utilities are in `src/threadpool.rs`.
- Utility modules include `src/time.rs`.
- A sweep algorithm for physics is implemented in `collision.rs`. 
- Tests are sperate `test.rs` files with `#[cfg(test)]` in the respected module.

## Build, Test, and Development Commands
- `cargo run` — launch the client with the default window and renderer.
- `cargo build --release` — optimized build for profiling/distribution.
- `cargo test` — run unit and integration tests.
- `cargo fmt` — format Rust sources; run before commits.
- `cargo clippy --all-targets --all-features` — lint for safety and idioms; treat warnings as actionable.
- `RUST_LOG=info cargo run` — enable structured logging via `env_logger`.

## Coding Style & Naming Conventions
- Follow rustfmt defaults (4-space indent, trailing commas, ordered `use` blocks).
- Modules/files use `snake_case`; types use `UpperCamelCase`; functions/locals use `snake_case`.
- Prefer small, focused modules under existing domains (`gpu`, `server`, `input`) rather than new top-level trees.
- Keep GPU resources (textures/shaders) close to their consumers in `src/gpu/`.

## Testing Guidelines
- For rendering or threading changes, add deterministic tests where possible and document any non-determinism.

## Commit & Pull Request Guidelines
- Use concise, imperative commit messages (e.g., `fix movement jitter`).
- PRs should describe behavior changes, performance impact, and test coverage.
- Include run commands and expected outputs (`cargo test`, `cargo clippy`).
- Provide screenshots or short clips for rendering changes; note seeds/settings used.

## Security & Configuration Tips
- Avoid embedding secrets; add configuration via environment variables.
- Prefer `Result` returns over panics in new I/O or threading paths.
- Keep dependencies current and favor pinned versions unless explicitly wildcarded.
