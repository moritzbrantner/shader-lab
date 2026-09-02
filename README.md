# shader-lab

A Rust + WebGPU laboratory for learning the graphics pipeline through small, inspectable experiments: shaders, buffers, transforms, textures, lighting, and later compute work.

## Scope

The native path uses `wgpu` and `winit`. Browser/WASM presentation is deliberately deferred until the native experiments are useful. The repository does not depend on Worldgen, Three-D, `rust-kernels`, `collision-lab`, or other `moritzbrantner/*` repositories during the initial four-PR horizon.

## Development

```sh
bash scripts/codex-environment.sh setup
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

Interactive rendering is a local/canary concern. The required deterministic gate must remain meaningful on machines without a physical GPU.
