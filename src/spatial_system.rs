use hecs::Entity;
use crate::spatial_grid::SpatialGrid;
use crate::quadtree::Quadtree;

/// High-performance spatial system that automatically chooses the best data structure
pub enum SpatialSystem {
    Grid(SpatialGrid),
    Quadtree(Quadtree),
}

impl SpatialSystem {
    /// Create a new spatial system optimized for the given entity count
    pub fn new(world_size: f32, entity_count: usize) -> Self {
        // Use quadtree for large numbers of entities (>1000)
        // Use grid for smaller numbers (better for small, dense populations)
        if entity_count > 1000 {
            let max_entities_per_node = (entity_count as f32).sqrt() as usize;
            let max_depth = 8;
            SpatialSystem::Quadtree(Quadtree::new(world_size, max_entities_per_node, max_depth))
        } else {
            // Optimize cell size based on entity density
            let cell_size = (world_size / (entity_count as f32).sqrt()).max(10.0).min(100.0);
            SpatialSystem::Grid(SpatialGrid::new(cell_size))
        }
    }

    pub fn clear(&mut self) {
        match self {
            SpatialSystem::Grid(grid) => grid.clear(),
            SpatialSystem::Quadtree(quadtree) => quadtree.clear(),
        }
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        match self {
            SpatialSystem::Grid(grid) => grid.insert(entity, x, y),
            SpatialSystem::Quadtree(quadtree) => quadtree.insert(entity, x, y),
        }
    }

    pub fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        match self {
            SpatialSystem::Grid(grid) => grid.get_nearby_entities(x, y, radius),
            SpatialSystem::Quadtree(quadtree) => quadtree.get_nearby_entities(x, y, radius),
        }
    }

    /// Optimized version with result limiting and pre-allocation
    pub fn get_nearby_entities_optimized(&self, x: f32, y: f32, radius: f32, limit: usize) -> Vec<Entity> {
        match self {
            SpatialSystem::Grid(grid) => {
                let mut results = grid.get_nearby_entities(x, y, radius);
                if results.len() > limit {
                    results.truncate(limit);
                    use rand::seq::SliceRandom;
                    use rand::thread_rng;
                    results.shuffle(&mut thread_rng());
                }
                results
            }
            SpatialSystem::Quadtree(quadtree) => quadtree.get_nearby_entities_optimized(x, y, radius, limit),
        }
    }

    /// Get the type of spatial system currently in use
    pub fn system_type(&self) -> &'static str {
        match self {
            SpatialSystem::Grid(_) => "Grid",
            SpatialSystem::Quadtree(_) => "Quadtree",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_system_grid() {
        let mut system = SpatialSystem::new(1000.0, 500);
        assert_eq!(system.system_type(), "Grid");
        
        let entity = hecs::Entity::new();
        system.insert(entity, 0.0, 0.0);
        
        let nearby = system.get_nearby_entities(0.0, 0.0, 10.0);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_spatial_system_quadtree() {
        let mut system = SpatialSystem::new(1000.0, 2000);
        assert_eq!(system.system_type(), "Quadtree");
        
        let entity = hecs::Entity::new();
        system.insert(entity, 0.0, 0.0);
        
        let nearby = system.get_nearby_entities(0.0, 0.0, 10.0);
        assert_eq!(nearby.len(), 1);
    }
} 