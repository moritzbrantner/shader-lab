# shader-lab context

## Goal

Learn and measure the GPU rendering pipeline by building progressively richer experiments whose source remains small enough to inspect.

## Vocabulary

**Shader** — GPU program executed at a pipeline stage. Initial experiments use WGSL vertex and fragment shaders.

**Pipeline** — Configured relationship between shader stages, vertex inputs, resources, rasterization, and render targets.

**Uniform** — Small read-only data supplied to shader invocations, such as transforms or time.

**Vertex buffer** — GPU buffer containing per-vertex data such as position, color, UVs, and normals.

**Canary** — Hardware/driver-dependent verification that is useful evidence but does not define deterministic repository green.

## Initial boundary

Native Rust is the first presentation surface. Browser/WASM work starts later, after the native concepts and experiment seams are stable enough to justify a second frontend.
