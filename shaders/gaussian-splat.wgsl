struct Camera {
    aspect: f32,
    focal: f32,
    near_plane: f32,
    far_plane: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct SplatInput {
    @location(0) center: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec3<f32>,
    @location(3) opacity: f32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) opacity: f32,
};

const QUAD: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(input: SplatInput) -> VertexOutput {
    let local = QUAD[input.vertex_index];
    let depth = max(-input.center.z, camera.near_plane);
    let ndc_center = vec2<f32>(
        input.center.x * camera.focal / (depth * camera.aspect),
        input.center.y * camera.focal / depth,
    );
    let radius_ndc = vec2<f32>(
        input.radius * camera.focal / (depth * camera.aspect),
        input.radius * camera.focal / depth,
    );
    let ndc = ndc_center + local * radius_ndc * 3.0;
    let depth_ndc = clamp(
        (depth - camera.near_plane) / (camera.far_plane - camera.near_plane),
        0.0,
        1.0,
    );

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, depth_ndc, 1.0);
    output.local = local * 3.0;
    output.color = input.color;
    output.opacity = input.opacity;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let gaussian = exp(-0.5 * dot(input.local, input.local));
    let alpha = input.opacity * gaussian;
    if alpha < 0.01 {
        discard;
    }
    return vec4<f32>(input.color, alpha);
}
