# Triangle experiment

## Question

What is the smallest native `wgpu` program that demonstrates the full vertex → rasterization → fragment path without hiding the pipeline behind an engine?

## Result

The application creates a native `winit` window, chooses a compatible adapter/surface configuration, compiles one WGSL module, and draws three vertices. The vertex positions are generated from `vertex_index`, so the first experiment needs no vertex buffer yet.

## Deterministic evidence

The required CI gate does not open a window or require a physical GPU. It parses the committed WGSL source with Naga and asserts that the expected vertex and fragment entry points exist. Interactive rendering remains local/canary evidence.

The next experiment will introduce explicit vertex buffers, uniforms, and transforms so GPU data layout becomes observable and testable.
