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
    @location(2) motion_shape: vec4<f32>,
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
    @builtin(instance_index) instance_index: u32,
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

    // Render creatures as soft bodies: keep them visible but avoid oversized
    // bright discs that blow out dense clusters.
    let screen_radius = clamp(radius / world_size * 2.0, 0.0055, 0.012) * uniforms.camera_zoom;

    // Keep a modest halo margin; the organism body now carries the read instead
    // of a wide glow cloud.
    let glow_extension = screen_radius * 0.75;
    let quad_size = screen_radius + glow_extension;

    let motion = curr_pos - prev_pos;
    let motion_len = length(motion);
    var motion_dir = vec2<f32>(1.0, 0.0);
    if (motion_len > 0.001) {
        motion_dir = vec2<f32>(motion.x, -motion.y) / motion_len;
    }
    let seed = fract(sin(f32(instance_index) * 12.9898 + radius * 37.719) * 43758.5453);

    out.position = vec4<f32>(screen_pos + quad_pos * quad_size, 0.0, 1.0);
    out.color = instance.radius_color.yzw;
    out.uv = quad_pos;  // -1 to 1 range
    out.motion_shape = vec4<f32>(motion_dir, clamp(motion_len * 0.4, 0.0, 1.0), seed);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = in.motion_shape.xy;
    let tangent = vec2<f32>(-dir.y, dir.x);
    let speed = in.motion_shape.z;
    let seed = in.motion_shape.w;

    // Stretch gently along the motion vector and add a low-frequency edge wobble
    // so organisms read as soft amoebas instead of perfect glowing discs.
    let along = dot(in.uv, dir) / (1.0 + speed * 0.18);
    let across = dot(in.uv, tangent) * (1.0 + speed * 0.08);
    let shaped_uv = vec2<f32>(along, across);
    let angle = atan2(shaped_uv.y, shaped_uv.x);
    let wobble =
        sin(angle * 3.0 + seed * 6.28318) * 0.08 +
        sin(angle * 5.0 - seed * 9.1) * 0.045;
    let dist = length(shaped_uv) / max(0.76, 1.0 + wobble);

    let body = smoothstep(1.04, 0.62, dist);
    let soft_edge = smoothstep(1.08, 0.86, dist);
    let inner = smoothstep(0.76, 0.16, dist);
    let halo = pow(max(0.0, 1.0 - dist), 2.0);

    let nucleus_offset = vec2<f32>(cos(seed * 6.28318), sin(seed * 6.28318)) * 0.18;
    let nucleus = smoothstep(0.26, 0.0, length(shaped_uv - nucleus_offset));
    let highlight_offset = vec2<f32>(-0.22, -0.28) + nucleus_offset * 0.28;
    let highlight = smoothstep(0.32, 0.0, length(shaped_uv - highlight_offset));

    let vivid = pow(max(in.color, vec3<f32>(0.001)), vec3<f32>(0.82));
    let cytoplasm = vivid * (body * 0.62 + inner * 0.32 + halo * 0.24);
    let membrane = mix(vivid, vec3<f32>(0.88, 0.96, 1.0), 0.38) * soft_edge * 0.28;
    let nucleus_rgb = mix(vivid, vec3<f32>(0.12, 0.16, 0.22), 0.38) * nucleus * 0.24;
    let highlight_rgb = vec3<f32>(1.0, 0.96, 0.84) * highlight * 0.18;
    let rgb = cytoplasm + membrane + nucleus_rgb + highlight_rgb;

    return vec4<f32>(rgb, body * 0.82);
}

// ---------------------------------------------------------------------------
// Food patches: a separate instanced draw of large, soft, dim teal blobs into
// the HDR scene BEFORE the creatures, so the viewer reads the food structure as
// ambient nourishment. Low brightness keeps it from competing with the bloom,
// and it shares the same world→screen + camera transform as the creatures.
// ---------------------------------------------------------------------------

struct FoodInstance {
    // x = world x, y = world y, z = radius (world units), w = intensity fraction (0..1)
    @location(0) data: vec4<f32>,
}

struct FoodVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) intensity: f32,
}

@vertex
fn food_vs(
    instance: FoodInstance,
    @builtin(vertex_index) vertex_index: u32,
) -> FoodVertexOutput {
    var out: FoodVertexOutput;

    let quad_pos = QUAD_VERTICES[vertex_index];
    let world_pos = instance.data.xy;
    let radius = instance.data.z;
    let world_size = uniforms.world_size;

    let world_to_screen_x = (world_pos.x + world_size / 2.0) / world_size * 2.0 - 1.0;
    let world_to_screen_y = -((world_pos.y + world_size / 2.0) / world_size * 2.0 - 1.0);
    let screen_x = (world_to_screen_x + uniforms.camera_x) * uniforms.camera_zoom;
    let screen_y = (world_to_screen_y + uniforms.camera_y) * uniforms.camera_zoom;
    let screen_pos = vec2<f32>(screen_x, screen_y);

    // Patches are large soft fields; map the world radius straight to screen.
    let screen_radius = radius / world_size * 2.0 * uniforms.camera_zoom;

    out.position = vec4<f32>(screen_pos + quad_pos * screen_radius, 0.0, 1.0);
    out.uv = quad_pos;
    out.intensity = instance.data.w;
    return out;
}

@fragment
fn food_fs(in: FoodVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    // Smooth radial falloff (squared) so the patch is a soft blob with no hard rim.
    let glow = max(0.0, 1.0 - dist);
    let soft = glow * glow;

    // Calm teal/green nourishment colour, kept dim so it reads as ambient food
    // and never competes with the creatures or the bloom. Scaled by the patch's
    // current intensity so a depleted patch visibly fades.
    let teal = vec3<f32>(0.07, 0.38, 0.30);
    let brightness = 0.58;
    let rgb = teal * soft * brightness * in.intensity;

    return vec4<f32>(rgb, soft);
}
