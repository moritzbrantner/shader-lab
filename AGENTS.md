# AGENTS.md

## Repository purpose

`shader-lab` is a learning and experiment repository for GPU rendering concepts. Prefer small numbered/explicit experiments and transparent graphics code over framework abstraction.

## Boundaries

- Keep native Rust + `wgpu` + WGSL as the reference implementation path for rendering experiments.
- GitHub Pages may use dependency-free browser WebGPU for explanations, small equivalent experiments, WGSL editing, and capability analysis when that makes the GPU boundary easier to inspect.
- Do not add React, Three.js, WASM, or another `moritzbrantner/*` dependency without a concrete experiment need. WASM should be introduced when sharing Rust implementation is valuable, not merely to bridge to the browser.
- Keep native and browser experiments conceptually aligned. When code is intentionally duplicated across presentation surfaces, keep the source relationship obvious enough that later divergence can be detected and repaired.
- Use an established math crate rather than turning this repository into a linear-algebra implementation project.
- Keep deterministic correctness checks independent of a physical GPU where practical: validate shader source and CPU-side layout/math without requiring display hardware.
- Hardware/driver performance evidence is canary evidence until a controlled benchmark environment exists.

## Validation

Run the narrowest affected checks first, then:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

Before interpreting native/GPU setup failures as product regressions, verify the declared environment fingerprint when `coding-tooling` is available.

Do not make required gates green by retrying deterministic failures. Do not use cross-machine pixel-perfect screenshots as a universal correctness oracle.

## Repository knowledge

- `CONTEXT.md` owns the concise graphics vocabulary and project boundary.
- Durable experiment notes belong under `docs/experiments/` when the experiment exists.
- Consequential, expensive-to-reverse architecture decisions belong under `docs/adr/`.
- TODOs must be actionable: `TODO: <action>` or `TODO(#123): <action>`.
