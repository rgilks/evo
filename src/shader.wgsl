// Uniforms for the simulation
struct SimulationUniforms {
    world_size: f32,
    interpolation_factor: f32,
    camera_zoom: f32,
    camera_x: f32,
    camera_y: f32,
    creature_scale: f32, // global creature-size multiplier ("Size" slider)
    padding2: f32,
    padding3: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: SimulationUniforms;

// Instance data: prev_pos (xy), curr_pos (xy), radius, color (rgb), and per-
// creature state (health + movement-style id) so the look reflects what the
// organism is *doing* and how it's faring.
struct InstanceInput {
    @location(0) prev_curr_pos: vec4<f32>, // xy = prev_pos, zw = curr_pos
    @location(1) radius_color: vec4<f32>, // x = radius, yzw = color (rgb)
    @location(2) state: vec4<f32>, // x=health(0..1), y=style id(0..4), z=speed gene(0..1), w=sense gene(0..1)
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) motion_shape: vec4<f32>,
    @location(3) state: vec4<f32>, // x=health, y=style id, z=speed gene, w=sense gene
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

    // Restore real size variation: a sqrt mapping spreads the [min,max] radius
    // range across a visible on-screen span so a giant predator dwarfs a runt,
    // while the clamp keeps dense clusters legible. Alpha-blended soft bodies
    // (not additive) stop overlaps from blowing out.
    let norm = radius / world_size * 2.0;
    let screen_radius = clamp(0.0034 + sqrt(max(norm, 0.0)) * 0.16, 0.005, 0.03)
        * uniforms.camera_zoom * uniforms.creature_scale;

    // A soft halo margin around the body; the bloom pass turns it into a
    // bioluminescent bleed for healthy creatures.
    let glow_extension = screen_radius * 0.9;
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
    out.state = instance.state;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = in.motion_shape.xy;
    let tangent = vec2<f32>(-dir.y, dir.x);
    let motion = in.motion_shape.z;          // 0..1 movement speed this frame
    let seed = in.motion_shape.w;
    let health = clamp(in.state.x, 0.0, 1.0);
    let style = in.state.y;
    let speed_gene = clamp(in.state.z, 0.0, 1.0);
    let sense_gene = clamp(in.state.w, 0.0, 1.0);

    // Body axes: along the motion vector, and across it.
    let along = dot(in.uv, dir);
    let across = dot(in.uv, tangent);

    // Streamlining: a fast genotype (and fast motion) narrows the body into a
    // dart; a slow one stays round. Width only ever shrinks, so the silhouette
    // stays inside the quad. This makes the speed gene visible as form.
    let streamline = clamp(speed_gene * 0.75 + motion * 0.25, 0.0, 1.0);
    var width = mix(1.0, 0.55, streamline);

    // Archetype by movement style — each behaviour gets its own body plan, so
    // speciation reads as visibly different creatures, not just colours:
    //   grazer = round & soft, predator = sharp dart, solitary = spiky star,
    //   flocker = streamlined, random = neutral.
    var spike_freq = 4.0;
    var spike_amp = 0.07;
    if (style > 3.5) {            // Grazing
        width = mix(width, 1.0, 0.6);
        spike_amp = 0.03;
    } else if (style > 2.5) {     // Predatory
        width = width * 0.82;
        spike_freq = 3.0;
        spike_amp = 0.05;
    } else if (style > 1.5) {     // Solitary
        spike_freq = 7.0;
        spike_amp = 0.20;
    } else if (style > 0.5) {     // Flocking
        width = width * 0.9;
        spike_freq = 5.0;
        spike_amp = 0.06;
    }

    // Teardrop: taper toward the front so a moving creature leads with a head.
    let taper = 1.0 - 0.35 * smoothstep(0.0, 0.9, along) * streamline;
    let eff_width = max(width * taper, 0.22);
    let shaped = vec2<f32>(along, across / eff_width);

    let angle = atan2(shaped.y, shaped.x);
    let wobble =
        sin(angle * spike_freq + seed * 6.28318) * spike_amp +
        sin(angle * (spike_freq + 2.0) - seed * 9.1) * spike_amp * 0.5;
    // Start spiky shapes from a smaller base so their points fit the quad.
    let base = 0.92 - spike_amp;
    let dist = length(shaped) / max(0.6, base + wobble);

    let body = smoothstep(1.04, 0.60, dist);
    let soft_edge = smoothstep(1.10, 0.84, dist);
    let inner = smoothstep(0.74, 0.12, dist);
    // Bioluminescent halo, widened for far-sighted (high-sense) genotypes so a
    // keen perceiver wears a visible aura — the sense gene made visible.
    let halo = pow(max(0.0, 1.0 - dist), 2.3) * (0.65 + sense_gene * 0.8);

    let nucleus_offset = vec2<f32>(cos(seed * 6.28318), sin(seed * 6.28318)) * 0.18;
    let nucleus = smoothstep(0.26, 0.0, length(shaped - nucleus_offset));
    let highlight_offset = vec2<f32>(-0.22, -0.28) + nucleus_offset * 0.28;
    let highlight = smoothstep(0.32, 0.0, length(shaped - highlight_offset));

    // Vitality drives luminance: a thriving cell glows brightly (and blooms), a
    // starving one dims to a faint ember. Newborns spawn at low energy and
    // brighten as they feed; the dying fade toward dark.
    let vitality = 0.34 + health * 1.02;
    // Predators (style id 3) carry a hotter, tighter nucleus — a hungry glint.
    let is_pred = step(2.5, style) * step(style, 3.5);

    let vivid = pow(max(in.color, vec3<f32>(0.001)), vec3<f32>(0.80)) * vitality;
    let cytoplasm = vivid * (body * 0.86 + inner * 0.50 + halo * 0.55);
    let membrane = mix(vivid, vec3<f32>(0.9, 0.98, 1.0), 0.40) * soft_edge * 0.42;
    let nucleus_tint = mix(vec3<f32>(0.12, 0.16, 0.22), vec3<f32>(1.0, 0.86, 0.6), is_pred);
    let nucleus_rgb = mix(vivid, nucleus_tint, 0.30) * nucleus * (0.28 + is_pred * 0.55);
    let highlight_rgb = vec3<f32>(1.0, 0.96, 0.84) * highlight * 0.26;
    let rgb = cytoplasm + membrane + nucleus_rgb + highlight_rgb;

    // Fade the whole organism with vitality so the dying dissolve into the dark.
    let alpha = body * (0.45 + health * 0.5);

    return vec4<f32>(rgb, alpha);
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
    @location(2) seed: f32,
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
    // A per-patch seed (from its world position) so each field wobbles its own
    // way instead of every patch being an identical disc.
    out.seed = fract(sin(dot(world_pos, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    return out;
}

@fragment
fn food_fs(in: FoodVertexOutput) -> @location(0) vec4<f32> {
    let intensity = in.intensity; // 0..1 fraction of the patch's capacity
    let angle = atan2(in.uv.y, in.uv.x);
    // Low-frequency organic wobble so the field reads as a living nutrient bloom
    // (algae/coral) rather than a perfect circle. Unique per patch via `seed`.
    let wobble =
        sin(angle * 3.0 + in.seed * 6.28318) * 0.06 +
        sin(angle * 5.0 - in.seed * 11.0) * 0.035;
    let dist = length(in.uv) / (1.0 + wobble);

    let glow = max(0.0, 1.0 - dist);
    let soft = glow * glow;       // broad soft halo
    let core = pow(glow, 4.0);    // tight inner core that blooms when rich

    // Lush patches glow warm green-gold; grazed-out ones cool to a faint teal, so
    // a glance tells you which fields are feeding the swarm and which are spent.
    let lush = vec3<f32>(0.24, 0.86, 0.42);
    let spent = vec3<f32>(0.05, 0.20, 0.24);
    let tint = mix(spent, lush, intensity);

    // Brightness tracks richness; a full patch's core pushes past the bloom
    // threshold so it reads as a luminous feeding ground.
    let field = soft * 0.45 + core * 1.05 * intensity;
    let rgb = tint * field;

    return vec4<f32>(rgb, soft);
}

// ---------------------------------------------------------------------------
// Transient effects: expanding glowing rings for predation flashes (kind 0),
// bloom/seed bursts (kind 1), and cull shockwaves (kind 2). Drawn additively
// into the HDR scene AFTER the creatures so they read as flashes on top. The
// ring animation is interpolated between sim ticks via the shared
// interpolation_factor, so it stays smooth even though effects age at the
// (slower) tick rate.
// ---------------------------------------------------------------------------

struct EffectInstance {
    // x = world x, y = world y, z = base radius (world), w = life (age/max_age, 0..1)
    @location(0) a: vec4<f32>,
    // x = life_step (1/max_age), y = kind (0..2)
    @location(1) b: vec2<f32>,
}

struct EffectVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) life: f32,
    @location(2) kind: f32,
}

@vertex
fn effect_vs(
    instance: EffectInstance,
    @builtin(vertex_index) vertex_index: u32,
) -> EffectVertexOutput {
    var out: EffectVertexOutput;

    let quad_pos = QUAD_VERTICES[vertex_index];
    let world_pos = instance.a.xy;
    let base_radius = instance.a.z;
    // Interpolate life between ticks for a smooth 60fps expansion.
    let life = clamp(instance.a.w + uniforms.interpolation_factor * instance.b.x, 0.0, 1.0);
    let world_size = uniforms.world_size;

    let world_to_screen_x = (world_pos.x + world_size / 2.0) / world_size * 2.0 - 1.0;
    let world_to_screen_y = -((world_pos.y + world_size / 2.0) / world_size * 2.0 - 1.0);
    let screen_x = (world_to_screen_x + uniforms.camera_x) * uniforms.camera_zoom;
    let screen_y = (world_to_screen_y + uniforms.camera_y) * uniforms.camera_zoom;
    let screen_pos = vec2<f32>(screen_x, screen_y);

    let screen_radius = base_radius / world_size * 2.0 * uniforms.camera_zoom;

    out.position = vec4<f32>(screen_pos + quad_pos * screen_radius, 0.0, 1.0);
    out.uv = quad_pos;
    out.life = life;
    out.kind = instance.b.y;
    return out;
}

@fragment
fn effect_fs(in: EffectVertexOutput) -> @location(0) vec4<f32> {
    let d = length(in.uv);
    let life = in.life;
    // A ring expanding from centre to rim over its life, widening as it goes.
    let r = life;
    let thickness = 0.10 + 0.12 * life;
    let ring = exp(-pow((d - r) / thickness, 2.0));
    // Dissolve as it reaches the rim.
    let fade = pow(1.0 - life, 1.5);
    let intensity = ring * fade;

    // Colour by kind: hot gold flash, green seed-burst, red cull ripple.
    var col = vec3<f32>(1.0, 0.85, 0.5);
    if (in.kind > 1.5) {
        col = vec3<f32>(1.0, 0.4, 0.45);
    } else if (in.kind > 0.5) {
        col = vec3<f32>(0.5, 1.0, 0.6);
    }

    let rgb = col * intensity * 1.7;
    return vec4<f32>(rgb, intensity);
}
