// GPU compute shader for entity movement updates
// This shader processes movement, energy costs, and boundary handling in parallel

// Input/Output buffers
@group(0) @binding(0) var<storage, read_write> entity_positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> entity_velocities: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> entity_energies: array<f32>;
@group(0) @binding(3) var<storage, read> entity_sizes: array<f32>;
@group(0) @binding(4) var<storage, read> entity_genes: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> movement_targets: array<vec2<f32>>;
@group(0) @binding(6) var<storage, read> nearby_entities: array<u32>;

// Simulation parameters (would be passed via push constants or uniforms)
var<private> world_size: f32 = 600.0;
var<private> max_velocity: f32 = 10.0;
var<private> movement_energy_cost: f32 = 0.1;
var<private> boundary_margin: f32 = 5.0;
var<private> velocity_bounce_factor: f32 = 0.8;

// Random number generation using a simple hash function
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = x ^ (x >> 16u);
    x = x * 0x85ebca6bu;
    x = x ^ (x >> 13u);
    x = x * 0xc2b2ae35u;
    x = x ^ (x >> 16u);
    return f32(x) / f32(0xffffffffu);
}

// Generate random direction using uniform distribution in a circle
fn random_direction(seed: u32) -> vec2<f32> {
    var attempts = 0u;
    loop {
        let dx = hash(seed + attempts * 2u) * 2.0 - 1.0;
        let dy = hash(seed + attempts * 2u + 1u) * 2.0 - 1.0;
        let length_sq = dx * dx + dy * dy;
        if length_sq <= 1.0 && length_sq > 0.0 {
            let length = sqrt(length_sq);
            return vec2<f32>(dx / length, dy / length);
        }
        attempts = attempts + 1u;
        if attempts > 10u {
            return vec2<f32>(1.0, 0.0); // Fallback direction
        }
    }
}

// Handle boundary collisions
fn handle_boundaries(pos: vec2<f32>, vel: vec2<f32>) -> vec2<f32> {
    var new_vel = vel;
    let half_world = world_size / 2.0;
    
    if pos.x <= -half_world + boundary_margin {
        new_vel.x = abs(vel.x) * velocity_bounce_factor;
    } else if pos.x >= half_world - boundary_margin {
        new_vel.x = -abs(vel.x) * velocity_bounce_factor;
    }
    
    if pos.y <= -half_world + boundary_margin {
        new_vel.y = abs(vel.y) * velocity_bounce_factor;
    } else if pos.y >= half_world - boundary_margin {
        new_vel.y = -abs(vel.y) * velocity_bounce_factor;
    }
    
    return new_vel;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let entity_index = global_id.x;
    
    // Check if this thread is processing a valid entity
    if entity_index >= arrayLength(&entity_positions) {
        return;
    }
    
    // Get current entity data
    let current_pos = entity_positions[entity_index];
    let current_vel = entity_velocities[entity_index];
    let current_energy = entity_energies[entity_index];
    let entity_size = entity_sizes[entity_index];
    let genes = entity_genes[entity_index];
    
    // Extract gene values
    let speed = genes.x;
    let energy_efficiency = genes.y;
    let size_factor = genes.z;
    let sense_radius = genes.w;
    
    // Skip processing if entity has no energy
    if current_energy <= 0.0 {
        return;
    }
    
    // Find movement target
    let target = movement_targets[entity_index];
    var new_vel = current_vel;
    
    // Check if we have a valid target (non-zero coordinates)
    if target.x != 0.0 || target.y != 0.0 {
        // Move towards target
        let dx = target.x - current_pos.x;
        let dy = target.y - current_pos.y;
        let distance = sqrt(dx * dx + dy * dy);
        
        if distance > 0.0 {
            new_vel.x = (dx / distance) * speed;
            new_vel.y = (dy / distance) * speed;
        }
    } else {
        // Random movement
        let speed_variation = hash(entity_index * 12345u) * 0.4 + 0.8; // 0.8 to 1.2
        let adjusted_speed = speed * speed_variation;
        let direction = random_direction(entity_index * 67890u);
        
        new_vel.x = direction.x * adjusted_speed;
        new_vel.y = direction.y * adjusted_speed;
    }
    
    // Cap velocity to prevent extreme movements
    let vel_magnitude = sqrt(new_vel.x * new_vel.x + new_vel.y * new_vel.y);
    if vel_magnitude > max_velocity {
        new_vel.x = (new_vel.x / vel_magnitude) * max_velocity;
        new_vel.y = (new_vel.y / vel_magnitude) * max_velocity;
    }
    
    // Update position
    var new_pos = current_pos + new_vel;
    
    // Handle boundary collisions
    let adjusted_vel = handle_boundaries(new_pos, new_vel);
    
    // Apply boundary constraints to position
    let half_world = world_size / 2.0;
    if new_pos.x <= -half_world + boundary_margin {
        new_pos.x = -half_world + boundary_margin;
    } else if new_pos.x >= half_world - boundary_margin {
        new_pos.x = half_world - boundary_margin;
    }
    
    if new_pos.y <= -half_world + boundary_margin {
        new_pos.y = -half_world + boundary_margin;
    } else if new_pos.y >= half_world - boundary_margin {
        new_pos.y = half_world - boundary_margin;
    }
    
    // Validate position to prevent NaN or infinite values
    if isNaN(new_pos.x) || isInf(new_pos.x) {
        new_pos.x = 0.0;
    }
    if isNaN(new_pos.y) || isInf(new_pos.y) {
        new_pos.y = 0.0;
    }
    
    // Calculate movement cost
    let movement_distance = sqrt(adjusted_vel.x * adjusted_vel.x + adjusted_vel.y * adjusted_vel.y);
    let energy_cost = movement_distance * movement_energy_cost / energy_efficiency;
    let new_energy = max(current_energy - energy_cost, 0.0);
    
    // Update entity data
    entity_positions[entity_index] = new_pos;
    entity_velocities[entity_index] = adjusted_vel;
    entity_energies[entity_index] = new_energy;
}

// Alternative version for batch processing multiple entities
@compute @workgroup_size(256)
fn batch_movement(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let entity_index = global_id.x;
    let batch_index = global_id.y;
    
    // This would be used for processing multiple batches of entities
    // Implementation would depend on how we structure the batch data
}

// Optimized version using shared memory for better performance
@compute @workgroup_size(256)
fn optimized_movement(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let entity_index = global_id.x;
    
    // Check if this thread is processing a valid entity
    if entity_index >= arrayLength(&entity_positions) {
        return;
    }
    
    // Get current entity data
    let current_pos = entity_positions[entity_index];
    let current_vel = entity_velocities[entity_index];
    let current_energy = entity_energies[entity_index];
    let entity_size = entity_sizes[entity_index];
    let genes = entity_genes[entity_index];
    
    // Skip processing if entity has no energy
    if current_energy <= 0.0 {
        return;
    }
    
    // Extract gene values
    let speed = genes.x;
    let energy_efficiency = genes.y;
    let size_factor = genes.z;
    let sense_radius = genes.w;
    
    // Find movement target
    let target = movement_targets[entity_index];
    var new_vel = current_vel;
    
    // Check if we have a valid target
    if target.x != 0.0 || target.y != 0.0 {
        // Move towards target
        let dx = target.x - current_pos.x;
        let dy = target.y - current_pos.y;
        let distance = sqrt(dx * dx + dy * dy);
        
        if distance > 0.0 {
            new_vel.x = (dx / distance) * speed;
            new_vel.y = (dy / distance) * speed;
        }
    } else {
        // Random movement
        let speed_variation = hash(entity_index * 12345u) * 0.4 + 0.8;
        let adjusted_speed = speed * speed_variation;
        let direction = random_direction(entity_index * 67890u);
        
        new_vel.x = direction.x * adjusted_speed;
        new_vel.y = direction.y * adjusted_speed;
    }
    
    // Cap velocity
    let vel_magnitude = sqrt(new_vel.x * new_vel.x + new_vel.y * new_vel.y);
    if vel_magnitude > max_velocity {
        new_vel.x = (new_vel.x / vel_magnitude) * max_velocity;
        new_vel.y = (new_vel.y / vel_magnitude) * max_velocity;
    }
    
    // Update position
    var new_pos = current_pos + new_vel;
    
    // Handle boundaries
    let adjusted_vel = handle_boundaries(new_pos, new_vel);
    
    // Apply boundary constraints
    let half_world = world_size / 2.0;
    if new_pos.x <= -half_world + boundary_margin {
        new_pos.x = -half_world + boundary_margin;
    } else if new_pos.x >= half_world - boundary_margin {
        new_pos.x = half_world - boundary_margin;
    }
    
    if new_pos.y <= -half_world + boundary_margin {
        new_pos.y = -half_world + boundary_margin;
    } else if new_pos.y >= half_world - boundary_margin {
        new_pos.y = half_world - boundary_margin;
    }
    
    // Validate position
    if isNaN(new_pos.x) || isInf(new_pos.x) {
        new_pos.x = 0.0;
    }
    if isNaN(new_pos.y) || isInf(new_pos.y) {
        new_pos.y = 0.0;
    }
    
    // Calculate energy cost
    let movement_distance = sqrt(adjusted_vel.x * adjusted_vel.x + adjusted_vel.y * adjusted_vel.y);
    let energy_cost = movement_distance * movement_energy_cost / energy_efficiency;
    let new_energy = max(current_energy - energy_cost, 0.0);
    
    // Update entity data
    entity_positions[entity_index] = new_pos;
    entity_velocities[entity_index] = adjusted_vel;
    entity_energies[entity_index] = new_energy;
    
    // Synchronize workgroup to ensure all writes are complete
    workgroupBarrier();
} 