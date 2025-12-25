// Uniforms for the simulation
struct SimulationUniforms {
    world_size: f32,
    interpolation_factor: f32,
    padding1: f32,
    padding2: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: SimulationUniforms;

// Instance data: prev_pos (xy), curr_pos (xy), radius, color (rgb)
struct InstanceInput {
    @location(0) prev_curr_pos: vec4<f32>, // xy = prev_pos, zw = curr_pos
    @location(1) radius_color: vec4<f32>, // x = radius, yzw = color (rgb)
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
    
    // GPU Interpolation
    let prev_pos = instance.prev_curr_pos.xy;
    let curr_pos = instance.prev_curr_pos.zw;
    let world_pos = mix(prev_pos, curr_pos, uniforms.interpolation_factor);

    let radius = instance.radius_color.x;
    let world_size = uniforms.world_size;
    
    // GPU Coordinate Transformation
    let screen_x = (world_pos.x + world_size / 2.0) / world_size * 2.0 - 1.0;
    let screen_y = -((world_pos.y + world_size / 2.0) / world_size * 2.0 - 1.0);
    let screen_pos = vec2<f32>(screen_x, screen_y);

    let screen_radius = (radius / world_size * 2.0 / 10.0); // Simplified scaling for vertex shader

    // Expand quad by radius with glow extension
    let glow_extension = screen_radius * 0.5;
    let quad_size = screen_radius + glow_extension;

    out.position = vec4<f32>(screen_pos + quad_pos * quad_size, 0.0, 1.0);
    out.color = instance.radius_color.yzw;
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