// Instance data: position (xy), radius, color (rgb)
struct InstanceInput {
    @location(0) pos_radius: vec4<f32>,  // xy = position, z = radius, w = unused
    @location(1) color: vec4<f32>,       // rgb = color, a = unused
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

// Quad vertices (generated in shader)
const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(
    instance: InstanceInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let quad_pos = QUAD_VERTICES[vertex_index];
    let screen_pos = instance.pos_radius.xy;
    let radius = instance.pos_radius.z;
    
    // Expand quad by radius with glow extension
    let glow_extension = radius * 0.5;
    let quad_size = radius + glow_extension;

    out.position = vec4<f32>(screen_pos + quad_pos * quad_size, 0.0, 1.0);
    out.color = instance.color.rgb;
    out.uv = quad_pos;  // -1 to 1 range

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center (uv is -1 to 1)
    let dist = length(in.uv);
    
    // Create glowing ball effect
    let core = smoothstep(1.0, 0.0, dist * 2.0);
    let glow_inner = smoothstep(1.0, 0.0, dist * 1.5) * 0.9;
    let glow_middle = smoothstep(1.0, 0.0, dist * 1.2) * 0.7;
    let glow_outer = smoothstep(1.0, 0.0, dist * 0.8) * 0.5;
    let glow_far = smoothstep(1.0, 0.0, dist * 0.5) * 0.3;

    let glow = core + glow_inner + glow_middle + glow_outer + glow_far;
    let alpha = glow * 0.95;
    let final_color = in.color * glow;
    
    // White glow for definition
    let white_glow = smoothstep(1.0, 0.0, dist * 0.4) * 0.2;
    let final_color_with_glow = final_color + vec3<f32>(white_glow);

    return vec4<f32>(final_color_with_glow, alpha);
}