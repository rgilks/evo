// Post-processing: bright-pass + separable Gaussian blur + tonemapped composite.
// All passes draw a single fullscreen triangle. The scene is rendered to an HDR
// (rgba16float) target; bloom is computed at reduced resolution and added back.

const THRESHOLD: f32 = 0.42;   // brightness above which a pixel blooms

// Live, user-tunable post params. Bound at group 3 so the same value is shared by
// the fade and composite passes without colliding with the texture/sampler
// bindings the blur passes use at groups 0/1. Written from postprocess.rs; see
// WebGpuRenderer::set_visual_params.
//   bloom             — bloom add-back strength ("Glow" slider)
//   trail_persistence — per-frame HDR scene retention ("Trails" slider): higher =
//                       longer comet tails (the fade pass writes alpha = 1 - this)
//   exposure          — tonemap exposure ("Brightness" slider)
struct PostParams {
    bloom: f32,
    trail_persistence: f32,
    exposure: f32,
    _pad: f32,
};
@group(3) @binding(0) var<uniform> post: PostParams;

// Ambient background depth. Instead of fading to flat #000, the composite adds a
// faint deep-blue/violet radial glow so the void reads as atmospheric. Kept
// subtle so it never washes out the creatures or competes with the bloom; the
// glow is brightest at the centre and falls to a near-black vignette at the
// corners. These are pre-tonemap HDR intensities.
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.20, 0.11, 0.45); // deep blue-violet
const AMBIENT_CENTER: f32 = 0.12;   // glow intensity at screen centre
const AMBIENT_EDGE: f32 = 0.015;    // residual glow at the corners (vignette floor)

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// group 0: the input being sampled (scene or a bloom buffer) + a linear sampler.
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
// group 1: the blurred bloom buffer (only used by the composite pass).
@group(1) @binding(0) var bloom_src: texture_2d<f32>;

@vertex
fn fs_vert(@builtin(vertex_index) vi: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[vi];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>(p.x * 0.5 + 0.5, 1.0 - (p.y * 0.5 + 0.5));
    return out;
}

// Downsample the HDR scene with a 4-tap box (reduces aliasing of thin bright
// features) and keep only the part above the threshold.
@fragment
fn bright_pass(in: VsOut) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src));
    var c = textureSample(src, samp, in.uv + texel * vec2<f32>(-0.5, -0.5)).rgb;
    c += textureSample(src, samp, in.uv + texel * vec2<f32>(0.5, -0.5)).rgb;
    c += textureSample(src, samp, in.uv + texel * vec2<f32>(-0.5, 0.5)).rgb;
    c += textureSample(src, samp, in.uv + texel * vec2<f32>(0.5, 0.5)).rgb;
    c *= 0.25;
    let bright = max(c - vec3<f32>(THRESHOLD), vec3<f32>(0.0));
    return vec4<f32>(bright, 1.0);
}

// 9-tap Gaussian along `dir` (one texel step per sample), reading `src`.
fn gaussian(uv: vec2<f32>, dir: vec2<f32>) -> vec3<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src));
    var weights = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    var result = textureSample(src, samp, uv).rgb * weights[0];
    for (var i = 1; i < 5; i = i + 1) {
        let offset = dir * texel * f32(i);
        result += textureSample(src, samp, uv + offset).rgb * weights[i];
        result += textureSample(src, samp, uv - offset).rgb * weights[i];
    }
    return result;
}

// Trail fade. Drawn fullscreen over the HDR scene with standard alpha blending
// BEFORE the particles each frame. The alpha multiplies what's already there:
//   scene_rgb = scene_rgb * (1 - alpha) = scene_rgb * TRAIL_PERSISTENCE
// so the previous frame decays by TRAIL_PERSISTENCE, leaving glowing trails that
// the bloom pass then picks up. The colour is black, so only the alpha matters.
@fragment
fn fade() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0 - post.trail_persistence);
}

@fragment
fn blur_h(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(gaussian(in.uv, vec2<f32>(1.0, 0.0)), 1.0);
}

@fragment
fn blur_v(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(gaussian(in.uv, vec2<f32>(0.0, 1.0)), 1.0);
}

// Add bloom + a subtle ambient background to the HDR scene, tonemap to LDR, and
// output to the swapchain.
@fragment
fn composite(in: VsOut) -> @location(0) vec4<f32> {
    let scene = textureSample(src, samp, in.uv).rgb;
    let bloom = textureSample(bloom_src, samp, in.uv).rgb;

    // Ambient depth: a soft radial glow brightest at the centre, easing to a faint
    // floor at the corners. `dist` is normalised so the screen centre is 0 and the
    // corners are ~1 regardless of aspect ratio.
    let dist = length(in.uv - vec2<f32>(0.5)) / length(vec2<f32>(0.5));
    let ambient = AMBIENT_COLOR * mix(AMBIENT_CENTER, AMBIENT_EDGE, dist * dist);

    let hdr = scene + bloom * post.bloom + ambient;
    var mapped = vec3<f32>(1.0) - exp(-hdr * post.exposure);

    // Gentle vignette draws the eye to the living centre without hiding creatures
    // near the edges (corners keep ~75% brightness).
    let vignette = smoothstep(1.3, 0.35, dist);
    mapped *= mix(0.7, 1.0, vignette);

    return vec4<f32>(mapped, 1.0);
}
