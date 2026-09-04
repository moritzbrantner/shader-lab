# Projected 3D Gaussian splats

## Question

What is the smallest native `wgpu` experiment that makes the core rendering idea behind 3D Gaussian Splatting visible without pretending to implement the full 3DGS paper pipeline?

## Result

`cargo run --example gaussian_splat` renders a small, fixed set of 3D Gaussian centers as instanced camera-facing quads. Each instance carries a world-space center, a world-space isotropic radius, RGB color, and opacity. The vertex shader projects the 3D center and radius into screen space; the fragment shader evaluates a Gaussian falloff and alpha-blends the result.

The sample data is intentionally sorted back-to-front on the CPU so standard alpha blending is deterministic. Resizing updates a tiny camera uniform containing aspect ratio, focal length, and clip distances.

## What this demonstrates

- bulk splat data stays in a GPU-oriented buffer rather than becoming one scene object per Gaussian;
- a six-vertex quad can be generated from `vertex_index`, leaving the instance buffer to describe the splats;
- world-space splat size becomes screen-space footprint through perspective projection;
- the fragment stage turns a rectangular primitive into a Gaussian footprint with `exp(-0.5 * r²)`;
- transparent splats require ordering or a different compositing strategy.

## Deliberate limits

This is a 3DGS rendering baseline, not a complete trained-scene renderer. It deliberately omits anisotropic 3D covariance, quaternion rotation, spherical-harmonic view-dependent color, `.ply`/trained-scene loading, frustum/LOD streaming, and GPU depth sorting/binning.

Those omissions are useful boundaries for later experiments. A sensible progression is:

1. anisotropic covariance projected into a 2D ellipse;
2. stable per-frame depth keys plus GPU sorting/binning;
3. spherical-harmonic color evaluation;
4. canonical splat asset loading and large-buffer streaming;
5. profiling and WebGPU/browser parity where the shared implementation justifies it.

## ECS boundary

If `ecs-lab` later consumes this work, one splat cloud should normally be one semantic ECS entity with transform, visibility, bounds, LOD/streaming state, and a renderer handle. Individual Gaussians remain bulk renderer data. That keeps ECS ownership useful without making millions of splats into millions of entities or adding a cross-repository dependency here prematurely.
