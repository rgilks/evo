use hecs::Entity;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

/// Optimized spatial grid for efficient neighbor finding
#[derive(Default)]
pub struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn get_cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        (cell_x, cell_y)
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
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
        let grid = SpatialGrid::new(25.0);

        // Test positive coordinates
        let (cell_x, cell_y) = grid.get_cell_coords(50.0, 75.0);
        assert_eq!(cell_x, 2); // 50 / 25 = 2
        assert_eq!(cell_y, 3); // 75 / 25 = 3

        // Test negative coordinates
        let (cell_x, cell_y) = grid.get_cell_coords(-25.0, -50.0);
        assert_eq!(cell_x, -1); // -25 / 25 = -1
        assert_eq!(cell_y, -2); // -50 / 25 = -2

        // Test zero coordinates
        let (cell_x, cell_y) = grid.get_cell_coords(0.0, 0.0);
        assert_eq!(cell_x, 0);
        assert_eq!(cell_y, 0);

        // Test fractional coordinates
        let (cell_x, cell_y) = grid.get_cell_coords(12.5, 37.5);
        assert_eq!(cell_x, 0); // 12.5 / 25 = 0.5, floor = 0
        assert_eq!(cell_y, 1); // 37.5 / 25 = 1.5, floor = 1
    }

    #[test]
    fn test_entity_insertion_and_retrieval() {
        let mut grid = SpatialGrid::new(25.0);
        let entity1 = Entity::from_bits(0x1000000000000001).unwrap();
        let entity2 = Entity::from_bits(0x1000000000000002).unwrap();

        // Insert entities in the same cell
        grid.insert(entity1, 50.0, 75.0);
        grid.insert(entity2, 60.0, 80.0);

        // Both should be found in nearby search
        let nearby = grid.get_nearby_entities(50.0, 75.0, 20.0);
        assert!(nearby.contains(&entity1));
        assert!(nearby.contains(&entity2));
        assert_eq!(nearby.len(), 2);
    }

    #[test]
    fn test_nearby_entities_search() {
        let mut grid = SpatialGrid::new(25.0);
        let entity1 = Entity::from_bits(0x1000000000000001).unwrap();
        let entity2 = Entity::from_bits(0x1000000000000002).unwrap();
        let entity3 = Entity::from_bits(0x1000000000000003).unwrap();

        // Insert entities in different cells
        grid.insert(entity1, 0.0, 0.0); // Cell (0, 0)
        grid.insert(entity2, 50.0, 50.0); // Cell (2, 2)
        grid.insert(entity3, 100.0, 100.0); // Cell (4, 4)

        // Search from center cell with small radius
        let nearby = grid.get_nearby_entities(50.0, 50.0, 10.0);
        assert!(nearby.contains(&entity2)); // Should find entity in center
        assert!(!nearby.contains(&entity1)); // Should not find entity far away
        assert!(!nearby.contains(&entity3)); // Should not find entity far away
    }

    #[test]
    fn test_grid_clear() {
        let mut grid = SpatialGrid::new(25.0);
        let entity = Entity::from_bits(0x1000000000000001).unwrap();

        grid.insert(entity, 50.0, 75.0);
        assert!(!grid.grid.is_empty());

        grid.clear();
        assert!(grid.grid.is_empty());

        let nearby = grid.get_nearby_entities(50.0, 75.0, 10.0);
        assert!(nearby.is_empty());
    }

    #[test]
    fn test_multiple_entities_same_cell() {
        let mut grid = SpatialGrid::new(25.0);
        let entities: Vec<Entity> = (0..5)
            .map(|i| Entity::from_bits(0x1000000000000001 + i).unwrap())
            .collect();

        // Insert multiple entities in the same cell
        for (i, entity) in entities.iter().enumerate() {
            grid.insert(*entity, 50.0 + i as f32, 75.0 + i as f32);
        }

        let nearby = grid.get_nearby_entities(50.0, 75.0, 50.0);
        assert_eq!(nearby.len(), 5);
        for entity in &entities {
            assert!(nearby.contains(entity));
        }
    }

    #[test]
    fn test_edge_case_coordinates() {
        let mut grid = SpatialGrid::new(25.0);
        let entity = Entity::from_bits(0x1000000000000001).unwrap();

        // Test very large coordinates
        grid.insert(entity, 1000000.0, 1000000.0);
        let nearby = grid.get_nearby_entities(1000000.0, 1000000.0, 10.0);
        assert!(nearby.contains(&entity));

        // Test very small coordinates
        let entity2 = Entity::from_bits(0x1000000000000002).unwrap();
        grid.insert(entity2, -1000000.0, -1000000.0);
        let nearby = grid.get_nearby_entities(-1000000.0, -1000000.0, 10.0);
        assert!(nearby.contains(&entity2));
    }

    #[test]
    fn test_different_cell_sizes() {
        let mut grid_small = SpatialGrid::new(10.0);
        let mut grid_large = SpatialGrid::new(100.0);
        let entity = Entity::from_bits(0x1000000000000001).unwrap();

        // Same coordinates, different cell sizes
        grid_small.insert(entity, 50.0, 50.0);
        grid_large.insert(entity, 50.0, 50.0);

        // Small cell size should have more precise cell coordinates
        let (cell_x_small, cell_y_small) = grid_small.get_cell_coords(50.0, 50.0);
        let (cell_x_large, cell_y_large) = grid_large.get_cell_coords(50.0, 50.0);

        assert_eq!(cell_x_small, 5); // 50 / 10 = 5
        assert_eq!(cell_y_small, 5);
        assert_eq!(cell_x_large, 0); // 50 / 100 = 0.5, floor = 0
        assert_eq!(cell_y_large, 0);
    }

    #[test]
    fn test_zero_radius_search() {
        let mut grid = SpatialGrid::new(25.0);
        let entity = Entity::from_bits(0x1000000000000001).unwrap();

        grid.insert(entity, 50.0, 75.0);

        // Search with zero radius should only find entities in the exact same cell
        let nearby = grid.get_nearby_entities(50.0, 75.0, 0.0);
        assert!(nearby.contains(&entity));

        // Search from different cell with zero radius
        let nearby = grid.get_nearby_entities(100.0, 100.0, 0.0);
        assert!(!nearby.contains(&entity));
    }

    #[test]
    fn test_spatial_grid_bias() {
        let mut grid = SpatialGrid::new(25.0);

        // Create entities in a grid pattern
        let grid_size = 10;
        let world_size = 100.0;
        let spacing = world_size / grid_size as f32;

        let mut entities = Vec::new();

        let mut world = World::new();
        for i in 0..grid_size {
            for j in 0..grid_size {
                let x = (i as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;
                let y = (j as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;

                let entity = world.spawn((
                    Position { x, y },
                    Velocity { x: 0.0, y: 0.0 },
                    Energy {
                        current: 100.0,
                        max: 100.0,
                    },
                    Size { radius: 5.0 },
                    Genes::new_random(&mut thread_rng()),
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                    },
                ));
                entities.push((entity, x, y));
                grid.insert(entity, x, y);
            }
        }

        // Test neighbor detection from different positions
        let test_positions = vec![
            (0.0, 0.0),     // Center
            (-25.0, -25.0), // Bottom-left
            (25.0, 25.0),   // Top-right
            (-25.0, 25.0),  // Top-left
            (25.0, -25.0),  // Bottom-right
        ];

        for (test_x, test_y) in test_positions {
            let nearby = grid.get_nearby_entities(test_x, test_y, 30.0);

            // Calculate center of nearby entities
            let mut total_x = 0.0;
            let mut total_y = 0.0;
            let mut count = 0;

            for &entity in &nearby {
                if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == entity) {
                    total_x += x;
                    total_y += y;
                    count += 1;
                }
            }

            if count > 0 {
                let center_x = total_x / count as f32;
                let center_y = total_y / count as f32;

                println!(
                    "Test pos ({:.1}, {:.1}): {} nearby, center ({:.1}, {:.1})",
                    test_x, test_y, count, center_x, center_y
                );

                // Check for bias relative to test position
                let bias_x = center_x - test_x;
                let bias_y = center_y - test_y;

                if bias_x.abs() > 5.0 || bias_y.abs() > 5.0 {
                    println!("SPATIAL GRID BIAS DETECTED: ({:.1}, {:.1})", bias_x, bias_y);
                }
            }
        }
    }

    #[test]
    fn test_spatial_grid_order_bias() {
        let mut grid = SpatialGrid::new(25.0);

        let mut world = World::new();
        // Create entities in a specific pattern to test order bias
        let mut entities = Vec::new();

        let positions = [
            (-25.0, -25.0), // Bottom-left
            (25.0, -25.0),  // Bottom-right
            (-25.0, 25.0),  // Top-left
            (25.0, 25.0),
        ];

        // Insert entities in a specific order
        for (x, y) in positions.iter() {
            let entity = world.spawn((
                Position { x: *x, y: *y },
                Velocity { x: 0.0, y: 0.0 },
                Energy {
                    current: 100.0,
                    max: 100.0,
                },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
            ));
            entities.push((entity, *x, *y));
            grid.insert(entity, *x, *y);
        }

        // Test neighbor detection from center
        let nearby = grid.get_nearby_entities(0.0, 0.0, 30.0);

        println!("Nearby entities from center (0,0):");
        for (i, &entity) in nearby.iter().enumerate() {
            if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == entity) {
                println!("  {}: ({:.1}, {:.1})", i, x, y);
            }
        }

        // Check if there's a consistent order bias
        if nearby.len() >= 4 {
            let first_entity = nearby[0];
            if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == first_entity) {
                println!("First entity found: ({:.1}, {:.1})", x, y);

                // Check if it's consistently from a particular quadrant
                if *x < 0.0 && *y < 0.0 {
                    println!("BIAS DETECTED: First entity is from bottom-left quadrant!");
                } else if *x > 0.0 && *y < 0.0 {
                    println!("BIAS DETECTED: First entity is from bottom-right quadrant!");
                } else if *x < 0.0 && *y > 0.0 {
                    println!("BIAS DETECTED: First entity is from top-left quadrant!");
                } else {
                    println!("BIAS DETECTED: First entity is from top-right quadrant!");
                }
            }
        }
    }
}
