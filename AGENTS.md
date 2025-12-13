# Repository Guidelines

## Project Structure & Module Organization
- Core entrypoint: `src/main.rs` wires the winit event loop, camera, GPU drawer, and server.
- Rendering stack: `src/gpu/` holds camera controllers, shaders, textures (`*.png`), and mesh/buffer helpers.
- World logic: `src/server/` contains voxel types, chunk handling, and world generation strategies.
- Input and console: `src/input/` filters events; `src/console/` manages the in-app console; `src/threader/` hosts the thread pool utilities.
- Playground and timing helpers live in `src/playground.rs` and `src/time.rs`.

## Build, Test, and Development Commands
- `cargo run` — launch the client with the default window and renderer.
- `cargo build --release` — optimized build for profiling or distribution.
- `cargo fmt` — enforce Rust formatting; run before commits.
- `cargo clippy --all-targets --all-features` — lint for safety and idioms; treat warnings as actionable.
- `cargo test` — run the test suite (add tests as you modify modules).
- `RUST_LOG=info cargo run` — enable structured logging via `env_logger`.

## Coding Style & Naming Conventions
- Follow rustfmt defaults (4-space indent, trailing commas, ordered `use` blocks).
- Modules/files use `snake_case`; types use `UpperCamelCase`; functions and locals use `snake_case`.
- Prefer small, focused modules under existing domains (`gpu`, `server`, `input`) rather than new top-level trees.
- Keep GPU resources (`*.png`, shader files) close to their consumers in `src/gpu/`.

## Testing Guidelines
- Place unit tests inline with modules using `#[cfg(test)] mod tests` and `test_`-prefixed functions.
- For integration tests, add files under `tests/` mirroring module names (e.g., `tests/server_world_gen.rs`).
- When changing rendering or threading code, add deterministic unit tests where possible (logic, math, scheduling) and document any non-determinism.

## Commit & Pull Request Guidelines
- Use concise, imperative commit messages (e.g., `fix movement jitter`, `add chunk meshing benchmarks`), consistent with current history.
- PRs should describe behavior changes, performance impacts, and test coverage; link issues if applicable.
- Include run commands and expected outputs for reviewers (e.g., `cargo test`, `cargo clippy` results).
- Provide screenshots or short clips for visible rendering changes; note seeds/settings used to reproduce.

## Security & Configuration Tips
- Avoid embedding secrets; configuration should come from environment variables when added.
- Keep dependencies current; prefer pinned versions unless explicitly wildcarded (e.g., `noise`, `num`).
- Validate new file I/O or threading paths for panic safety and resource cleanup; use `Result` returns over panics in new code.
