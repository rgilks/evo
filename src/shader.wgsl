struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) center: vec2<f32>,
    @location(3) radius: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) center: vec2<f32>,
    @location(2) radius: f32,
    @location(3) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = vertex.color;
    out.center = vertex.center;
    out.radius = vertex.radius;
    
    // Calculate UV coordinates relative to the ball's center
    out.uv = (vertex.position - vertex.center) / vertex.radius;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center of the ball
    let dist = length(in.uv);
    
    // Create a proper circular shape with smooth falloff
    let circle = smoothstep(1.0, 0.0, dist);
    
    // Create a more pronounced glowing effect with multiple layers
    let core = smoothstep(1.0, 0.0, dist * 2.0); // Bright core
    let glow_inner = smoothstep(1.0, 0.0, dist * 1.5) * 0.9; // Inner glow
    let glow_middle = smoothstep(1.0, 0.0, dist * 1.2) * 0.7; // Middle glow
    let glow_outer = smoothstep(1.0, 0.0, dist * 0.8) * 0.5; // Outer glow
    let glow_far = smoothstep(1.0, 0.0, dist * 0.5) * 0.3; // Far glow
    
    // Combine all glow layers for a more intense effect
    let glow = core + glow_inner + glow_middle + glow_outer + glow_far;
    
    // Create the final color with transparency
    let alpha = glow * 0.95; // More opaque for better visibility
    let final_color = in.color * glow;
    
    // Add a subtle white glow around the edges for better definition
    let white_glow = smoothstep(1.0, 0.0, dist * 0.4) * 0.2;
    let final_color_with_glow = final_color + vec3<f32>(white_glow);
    
    return vec4<f32>(final_color_with_glow, alpha);
} 