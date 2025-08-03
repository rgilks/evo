// GPU compute shader for spatial queries
// This shader finds all entities within a given radius of a query point

// Input buffers
@group(0) @binding(0) var<storage, read> entity_positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> entity_radii: array<f32>;
@group(0) @binding(2) var<storage, read> entity_ids: array<u32>;

// Output buffers
@group(0) @binding(3) var<storage, read_write> query_results: array<u32>;
@group(0) @binding(4) var<storage, read_write> query_count: array<atomic<u32>>;

// Uniform buffer for query parameters
@group(0) @binding(5) var<uniform> query_params: vec3<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let entity_index = global_id.x;
    
    // Check if this thread should process an entity
    if entity_index >= arrayLength(&entity_positions) {
        return;
    }
    
    // Get entity data
    let entity_pos = entity_positions[entity_index];
    let entity_radius = entity_radii[entity_index];
    let entity_id = entity_ids[entity_index];
    
    // Calculate distance to query point
    let dx = entity_pos.x - query_params.x;
    let dy = entity_pos.y - query_params.y;
    let distance_squared = dx * dx + dy * dy;
    let radius_squared = (query_params.z + entity_radius) * (query_params.z + entity_radius);
    
    // Check if entity is within range
    if distance_squared <= radius_squared {
        // Add entity to results using atomic operation
        let result_index = atomicAdd(&query_count[0], 1u);
        if result_index < arrayLength(&query_results) {
            query_results[result_index] = entity_id;
        }
    }
} 