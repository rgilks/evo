use hecs::Entity;
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
        self.grid.entry(cell).or_insert_with(Vec::new).push(entity);
    }

    pub fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut nearby = Vec::new();
        let center_cell = self.get_cell_coords(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(entities) = self.grid.get(&cell) {
                    nearby.extend(entities.iter().copied());
                }
            }
        }

        nearby
    }


}

#[cfg(test)]
mod tests {
    use super::*;

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
}
