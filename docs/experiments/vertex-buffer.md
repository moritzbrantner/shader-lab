# Vertex-buffer triangle

This experiment replaces positions synthesized from `@builtin(vertex_index)` with bytes supplied by the CPU through a real GPU vertex buffer.

Each vertex is five consecutive `f32` values: two values for position followed by three values for color. The render pipeline declares that byte contract with a 20-byte stride, `Float32x2` at shader location 0, and `Float32x3` at shader location 1. WGSL receives those attributes as a typed `VertexInput`, forwards color through the vertex stage, and interpolates it across the triangle for the fragment stage.

Run it with:

```sh
cargo run --example vertex_buffer
```

## What to inspect

- The buffer itself is only bytes; `VertexBufferLayout` tells the pipeline how to interpret those bytes.
- `set_vertex_buffer(0, ...)` binds the allocation to vertex-buffer slot 0 before the draw call.
- `@location(0)` and `@location(1)` are the contract between the Rust-side layout and the WGSL vertex inputs.
- Color is emitted by each vertex and interpolated automatically before the fragment shader receives it.

This slice intentionally stops before uniforms and transforms. Those are the next experiment, where data changes per draw rather than per vertex.
