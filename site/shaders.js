const sharedVertex = `
struct Params {
    resolution: vec2<f32>,
    time: f32,
    speed: f32,
    pointer: vec2<f32>,
    padding: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let position = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = position * 0.5 + vec2<f32>(0.5);
    return output;
}
`;

const fragments = {
  gradient: `
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pulse = 0.5 + 0.5 * sin(params.time * params.speed + input.uv.x * 6.283185);
    return vec4<f32>(input.uv.x, input.uv.y, pulse, 1.0);
}
`,
  rings: `
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let safe_height = max(params.resolution.y, 1.0);
    let aspect = params.resolution.x / safe_height;
    let p = (input.uv - vec2<f32>(0.5)) * vec2<f32>(aspect, 1.0);
    let radius = length(p);
    let wave = 0.5 + 0.5 * cos(radius * 42.0 - params.time * params.speed * 4.0);
    let fade = exp(-radius * 2.4);
    return vec4<f32>(0.08 + wave * 0.22, 0.18 + wave * 0.52, 0.34 + fade * 0.62, 1.0);
}
`,
  pointer: `
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let p = input.uv - params.pointer;
    let distance_to_pointer = length(p);
    let glow = exp(-distance_to_pointer * 14.0);
    let stripes = 0.5 + 0.5 * sin((input.uv.x + input.uv.y) * 28.0 + params.time * params.speed * 2.0);
    return vec4<f32>(0.08 + glow * 0.82, 0.12 + stripes * 0.28, 0.22 + glow * 0.55, 1.0);
}
`,
  cells: `
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let grid = input.uv * 12.0;
    let cell = floor(grid);
    let local = fract(grid) - vec2<f32>(0.5);
    let parity = f32((i32(cell.x) + i32(cell.y)) & 1);
    let circle = smoothstep(0.38, 0.30, length(local));
    let phase = 0.5 + 0.5 * sin(params.time * params.speed + cell.x * 0.7 + cell.y * 0.45);
    let base = mix(vec3<f32>(0.04, 0.06, 0.10), vec3<f32>(0.11, 0.17, 0.25), parity);
    let accent = vec3<f32>(0.16, 0.62, 0.95) * circle * phase;
    return vec4<f32>(base + accent, 1.0);
}
`,
};

export const presets = [
  {
    id: "gradient",
    label: "UV gradient",
    question: "How do interpolated coordinates become fragment color?",
    source: `${sharedVertex}${fragments.gradient}`,
  },
  {
    id: "rings",
    label: "Animated rings",
    question: "How do uniforms make one shader respond to time and resolution?",
    source: `${sharedVertex}${fragments.rings}`,
  },
  {
    id: "pointer",
    label: "Pointer field",
    question: "How does CPU-side input become data consumed by every fragment?",
    source: `${sharedVertex}${fragments.pointer}`,
  },
  {
    id: "cells",
    label: "Procedural cells",
    question: "How much visible structure can a fragment shader create without textures?",
    source: `${sharedVertex}${fragments.cells}`,
  },
];

export function presetById(id) {
  return presets.find((preset) => preset.id === id) ?? presets[0];
}
