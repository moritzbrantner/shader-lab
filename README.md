# shader-lab

A Rust + WebGPU laboratory for learning the graphics pipeline through small, inspectable experiments: shaders, buffers, transforms, textures, lighting, and later compute work.

## Scope

Native Rust with `wgpu` and `winit` remains the reference implementation path. GitHub Pages is the companion learning surface: it may use browser WebGPU directly for explanations, small equivalent experiments, WGSL editing, and adapter analysis without hiding the GPU contracts being taught.

Keep the browser path lightweight. React, Three.js, WASM, and dependencies on other `moritzbrantner/*` repositories should be added only when an experiment has a concrete need for them. In particular, WASM should earn its place by sharing useful Rust implementation rather than existing only as a browser bridge.

## Development

```sh
bash scripts/codex-environment.sh setup
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

The static Pages lab can be served locally without a build step:

```sh
python3 -m http.server 8000 --directory site
```

Interactive rendering is a local/canary concern. The required deterministic gate must remain meaningful on machines without a physical GPU.

## Native experiments

```sh
cargo run --example vertex_buffer
cargo run --example gaussian_splat
```

The Gaussian-splat experiment is intentionally a small 3DGS rendering baseline: projected 3D centers, perspective-scaled Gaussian footprints, deterministic back-to-front alpha compositing, and explicit GPU instance data. Full covariance projection, view-dependent spherical harmonics, large-scene loading, and GPU sorting remain later experiments rather than hidden complexity in the first slice.

## Pages lab

The public lab follows three explicit surfaces:

- `site/explain/` builds the rendering-pipeline mental model.
- `site/demo/` compiles and renders editable WGSL through browser WebGPU.
- `site/analysis/` reports machine-specific adapter features and limits as canary evidence.
