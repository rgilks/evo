// Uniforms for the simulation
struct SimulationUniforms {
    world_size: f32,
    interpolation_factor: f32,
    camera_zoom: f32,
    camera_x: f32,
    camera_y: f32,
    padding1: f32,
    padding2: f32,
    padding3: f32,
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
    // World to Screen transformation
    let world_to_screen_x = (world_pos.x + world_size / 2.0) / world_size * 2.0 - 1.0;
    let world_to_screen_y = -((world_pos.y + world_size / 2.0) / world_size * 2.0 - 1.0);
    
    // Apply camera transformation (pan and zoom)
    let screen_x = (world_to_screen_x + uniforms.camera_x) * uniforms.camera_zoom;
    let screen_y = (world_to_screen_y + uniforms.camera_y) * uniforms.camera_zoom;
    let screen_pos = vec2<f32>(screen_x, screen_y);

    // Render creatures as visible glowing discs: clamp on-screen size to a tight
    // band so the smallest still reads as a dot and the largest never becomes a
    // giant blob.
    let screen_radius = clamp(radius / world_size * 2.0, 0.005, 0.011) * uniforms.camera_zoom;

    // Expand quad by radius with glow extension. A wide extension gives the soft
    // halo room to fall off, so each creature throws a lush glow into the bloom.
    let glow_extension = screen_radius * 1.5;
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

    // Additive glow: a tight bright core plus a soft wide halo. Additive blending
    // sums these into the HDR scene target, so overlapping creatures build cores
    // that exceed 1.0 and feed the bloom pass.
    let glow = max(0.0, 1.0 - dist);
    let halo = glow * glow;
    let core = pow(glow, 5.0);
    let rgb = in.color * (halo * 1.0 + core * 2.6);

    return vec4<f32>(rgb, halo);
}