# shader-lab context

## Goal

Learn and measure the GPU rendering pipeline by building progressively richer experiments whose source remains small enough to inspect.

## Vocabulary

**Shader** — GPU program executed at a pipeline stage. Initial experiments use WGSL vertex and fragment shaders.

**Pipeline** — Configured relationship between shader stages, vertex inputs, resources, rasterization, and render targets.

**Uniform** — Small read-only data supplied to shader invocations, such as transforms or time.

**Vertex buffer** — GPU buffer containing per-vertex data such as position, color, UVs, and normals.

**Canary** — Hardware/driver-dependent verification that is useful evidence but does not define deterministic repository green.

## Presentation surfaces

Native Rust with `wgpu` is the reference implementation path. The GitHub Pages lab is a companion presentation surface that can use browser WebGPU directly to explain the same contracts, run small visible experiments, edit WGSL, and inspect adapter capabilities.

The browser path should remain transparent rather than becoming a second rendering framework. WASM comes later only when reusing Rust implementation across native and browser surfaces creates enough value to justify the additional build and runtime boundary.
