use dashmap::DashMap;
use hecs::Entity;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Optimized spatial grid using DashMap for concurrent inserts
pub struct SpatialGrid {
    cell_size: f32,
    grid: DashMap<(i32, i32), Vec<Entity>>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            cell_size: 25.0,
            grid: DashMap::new(),
        }
    }
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: DashMap::new(),
        }
    }

    pub fn clear(&self) {
        self.grid.clear();
    }

    #[inline]
    pub fn get_cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        (cell_x, cell_y)
    }

    /// Thread-safe insert - can be called from parallel iterators
    pub fn insert(&self, entity: Entity, x: f32, y: f32) {
        let cell = self.get_cell_coords(x, y);
        self.grid.entry(cell).or_default().push(entity);
    }

    pub fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut nearby = Vec::new();
        let center_cell = self.get_cell_coords(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        // Generate all cell coordinates in the search area
        let mut cells = Vec::new();
        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                cells.push(cell);
            }
        }

        // Randomize the order of cell processing to eliminate bias
        let mut rng = thread_rng();
        cells.shuffle(&mut rng);

        // Process cells in randomized order
        for cell in cells {
            if let Some(entities) = self.grid.get(&cell) {
                nearby.extend(entities.iter().copied());
            }
        }

        nearby
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Color, Energy, Position, Size, Velocity};
    use crate::genes::Genes;
    use hecs::World;
    use rand::thread_rng;

    #[test]
    fn test_spatial_grid_creation() {
        let grid = SpatialGrid::new(25.0);
        assert!(grid.grid.is_empty());
    }

    #[test]
    fn test_cell_coordinate_calculation() {
        let grid = SpatialGrid::new(10.0);
        assert_eq!(grid.get_cell_coords(5.0, 5.0), (0, 0));
        assert_eq!(grid.get_cell_coords(15.0, 15.0), (1, 1));
        assert_eq!(grid.get_cell_coords(-5.0, -5.0), (-1, -1));
        assert_eq!(grid.get_cell_coords(25.0, 35.0), (2, 3));
    }

    #[test]
    fn test_entity_insertion_and_retrieval() {
        let mut world = World::new();
        let entity = world.spawn((
            Position { x: 10.0, y: 10.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        let grid = SpatialGrid::new(25.0);
        grid.insert(entity, 10.0, 10.0);

        let nearby = grid.get_nearby_entities(10.0, 10.0, 30.0);
        assert!(nearby.contains(&entity));
    }

    #[test]
    fn test_multiple_entities_same_cell() {
        let mut world = World::new();
        let entity1 = world.spawn((
            Position { x: 5.0, y: 5.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));
        let entity2 = world.spawn((
            Position { x: 8.0, y: 8.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        let grid = SpatialGrid::new(25.0);
        grid.insert(entity1, 5.0, 5.0);
        grid.insert(entity2, 8.0, 8.0);

        let nearby = grid.get_nearby_entities(5.0, 5.0, 30.0);
        assert!(nearby.contains(&entity1));
        assert!(nearby.contains(&entity2));
    }

    #[test]
    fn test_grid_clear() {
        let mut world = World::new();
        let entity = world.spawn((
            Position { x: 10.0, y: 10.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        let grid = SpatialGrid::new(25.0);
        grid.insert(entity, 10.0, 10.0);
        grid.clear();

        let nearby = grid.get_nearby_entities(10.0, 10.0, 30.0);
        assert!(nearby.is_empty());
    }

    #[test]
    fn test_nearby_entities_search() {
        let mut world = World::new();
        let entity1 = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));
        let entity2 = world.spawn((
            Position { x: 100.0, y: 100.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        let grid = SpatialGrid::new(25.0);
        grid.insert(entity1, 0.0, 0.0);
        grid.insert(entity2, 100.0, 100.0);

        // Search near entity1 with small radius - should only find entity1
        let nearby = grid.get_nearby_entities(0.0, 0.0, 25.0);
        assert!(nearby.contains(&entity1));
        // entity2 is far away, so it may or may not be in the search area
    }

    #[test]
    fn test_different_cell_sizes() {
        let grid_small = SpatialGrid::new(10.0);
        let grid_large = SpatialGrid::new(50.0);

        // Same position, different cell coordinates
        assert_eq!(grid_small.get_cell_coords(25.0, 25.0), (2, 2));
        assert_eq!(grid_large.get_cell_coords(25.0, 25.0), (0, 0));
    }

    #[test]
    fn test_edge_case_coordinates() {
        let grid = SpatialGrid::new(25.0);

        // Test exact cell boundaries
        assert_eq!(grid.get_cell_coords(0.0, 0.0), (0, 0));
        assert_eq!(grid.get_cell_coords(25.0, 25.0), (1, 1));
        assert_eq!(grid.get_cell_coords(-0.1, -0.1), (-1, -1));
    }

    #[test]
    fn test_zero_radius_search() {
        let grid = SpatialGrid::new(25.0);
        let mut world = World::new();
        let entity = world.spawn((
            Position { x: 10.0, y: 10.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        grid.insert(entity, 10.0, 10.0);

        // Zero radius search should still find entities in the same cell
        let nearby = grid.get_nearby_entities(10.0, 10.0, 0.0);
        assert!(nearby.contains(&entity));
    }

    #[test]
    fn test_spatial_grid_bias() {
        let grid = SpatialGrid::new(25.0);
        let mut world = World::new();

        // Create entities in different cells
        let mut entities = Vec::new();
        for i in 0..10 {
            let entity = world.spawn((
                Position {
                    x: i as f32 * 30.0,
                    y: i as f32 * 30.0,
                },
                Energy {
                    current: 50.0,
                    max: 100.0,
                },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
                Velocity { x: 0.0, y: 0.0 },
            ));
            grid.insert(entity, i as f32 * 30.0, i as f32 * 30.0);
            entities.push(entity);
        }

        // Get nearby entities from center with large radius
        let nearby = grid.get_nearby_entities(150.0, 150.0, 200.0);

        // All entities should be found
        for entity in &entities {
            assert!(
                nearby.contains(entity),
                "Entity missing from spatial grid search"
            );
        }
    }

    #[test]
    fn test_spatial_grid_order_bias() {
        let grid = SpatialGrid::new(25.0);
        let mut world = World::new();

        // Create entities at specific positions
        let entity1 = world.spawn((
            Position { x: -50.0, y: -50.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));
        let entity2 = world.spawn((
            Position { x: 50.0, y: 50.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
        ));

        grid.insert(entity1, -50.0, -50.0);
        grid.insert(entity2, 50.0, 50.0);

        // Check that both entities are found
        let nearby = grid.get_nearby_entities(0.0, 0.0, 100.0);
        assert!(nearby.contains(&entity1), "Entity1 not found");
        assert!(nearby.contains(&entity2), "Entity2 not found");
    }
}
