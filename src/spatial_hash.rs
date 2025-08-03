use hecs::Entity;
use std::collections::HashMap;

/// High-performance spatial hashing for million-scale simulations
pub struct SpatialHash {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Entity>>,
    entity_positions: HashMap<Entity, (f32, f32)>,
    max_entities_per_cell: usize,
}

impl SpatialHash {
    pub fn new(cell_size: f32, max_entities_per_cell: usize) -> Self {
        Self {
            cell_size,
            grid: HashMap::with_capacity(10000), // Pre-allocate for large worlds
            entity_positions: HashMap::with_capacity(100000), // Pre-allocate for many entities
            max_entities_per_cell,
        }
    }

    /// Convert world coordinates to grid coordinates
    fn world_to_grid(&self, x: f32, y: f32) -> (i32, i32) {
        let grid_x = (x / self.cell_size).floor() as i32;
        let grid_y = (y / self.cell_size).floor() as i32;
        (grid_x, grid_y)
    }

    /// Get all grid cells that intersect with a circle
    fn get_intersecting_cells(&self, center_x: f32, center_y: f32, radius: f32) -> Vec<(i32, i32)> {
        let (center_grid_x, center_grid_y) = self.world_to_grid(center_x, center_y);
        let radius_in_cells = (radius / self.cell_size).ceil() as i32;

        let mut cells = Vec::new();
        for dx in -radius_in_cells..=radius_in_cells {
            for dy in -radius_in_cells..=radius_in_cells {
                let grid_x = center_grid_x + dx;
                let grid_y = center_grid_y + dy;

                // Check if this cell actually intersects with the circle
                let cell_center_x = (grid_x as f32 + 0.5) * self.cell_size;
                let cell_center_y = (grid_y as f32 + 0.5) * self.cell_size;
                let distance = ((cell_center_x - center_x).powi(2)
                    + (cell_center_y - center_y).powi(2))
                .sqrt();

                if distance <= radius + self.cell_size * 0.707 {
                    // 0.707 = sqrt(2)/2
                    cells.push((grid_x, grid_y));
                }
            }
        }
        cells
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.entity_positions.clear();
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let grid_pos = self.world_to_grid(x, y);

        // Update entity position
        self.entity_positions.insert(entity, (x, y));

        // Add to grid cell
        self.grid
            .entry(grid_pos)
            .or_insert_with(Vec::new)
            .push(entity);
    }

    pub fn update_position(&mut self, entity: Entity, new_x: f32, new_y: f32) {
        if let Some((old_x, old_y)) = self.entity_positions.get(&entity) {
            let old_grid = self.world_to_grid(*old_x, *old_y);
            let new_grid = self.world_to_grid(new_x, new_y);

            if old_grid != new_grid {
                // Remove from old cell
                if let Some(cell) = self.grid.get_mut(&old_grid) {
                    cell.retain(|&e| e != entity);
                    if cell.is_empty() {
                        self.grid.remove(&old_grid);
                    }
                }

                // Add to new cell
                self.grid
                    .entry(new_grid)
                    .or_insert_with(Vec::new)
                    .push(entity);
            }
        }

        // Update position
        self.entity_positions.insert(entity, (new_x, new_y));
    }

    pub fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let intersecting_cells = self.get_intersecting_cells(x, y, radius);
        let radius_sq = radius * radius;

        let mut candidates = Vec::new();

        for cell_pos in intersecting_cells {
            if let Some(cell) = self.grid.get(&cell_pos) {
                for &entity in cell {
                    if let Some((entity_x, entity_y)) = self.entity_positions.get(&entity) {
                        let distance_sq = (entity_x - x).powi(2) + (entity_y - y).powi(2);
                        if distance_sq <= radius_sq {
                            candidates.push((entity, distance_sq));
                        }
                    }
                }
            }
        }

        // Sort by distance and limit results
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.into_iter().map(|(entity, _)| entity).collect()
    }

    /// Optimized version with result limiting
    pub fn get_nearby_entities_optimized(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        limit: usize,
    ) -> Vec<Entity> {
        let intersecting_cells = self.get_intersecting_cells(x, y, radius);
        let radius_sq = radius * radius;

        let mut candidates = Vec::new();

        for cell_pos in intersecting_cells {
            if let Some(cell) = self.grid.get(&cell_pos) {
                for &entity in cell {
                    if let Some((entity_x, entity_y)) = self.entity_positions.get(&entity) {
                        let distance_sq = (entity_x - x).powi(2) + (entity_y - y).powi(2);
                        if distance_sq <= radius_sq {
                            candidates.push((entity, distance_sq));
                            if candidates.len() >= limit * 2 {
                                // Get extra candidates for better sampling
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Sort by distance and limit results
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates
            .into_iter()
            .take(limit)
            .map(|(entity, _)| entity)
            .collect()
    }

    /// Batch insert multiple entities efficiently
    pub fn batch_insert(&mut self, entities: &[(Entity, f32, f32)]) {
        // Pre-allocate space
        let mut cell_updates: HashMap<(i32, i32), Vec<Entity>> = HashMap::new();

        for &(entity, x, y) in entities {
            let grid_pos = self.world_to_grid(x, y);
            cell_updates
                .entry(grid_pos)
                .or_insert_with(Vec::new)
                .push(entity);
            self.entity_positions.insert(entity, (x, y));
        }

        // Merge with existing grid
        for (grid_pos, new_entities) in cell_updates {
            self.grid
                .entry(grid_pos)
                .or_insert_with(Vec::new)
                .extend(new_entities);
        }
    }

    /// Get statistics about the spatial hash
    pub fn get_stats(&self) -> SpatialHashStats {
        let total_entities = self.entity_positions.len();
        let total_cells = self.grid.len();
        let max_cell_size = self.grid.values().map(|cell| cell.len()).max().unwrap_or(0);
        let avg_cell_size = if total_cells > 0 {
            total_entities as f32 / total_cells as f32
        } else {
            0.0
        };

        SpatialHashStats {
            total_entities,
            total_cells,
            max_cell_size,
            avg_cell_size,
            cell_size: self.cell_size,
        }
    }
}

#[derive(Debug)]
pub struct SpatialHashStats {
    pub total_entities: usize,
    pub total_cells: usize,
    pub max_cell_size: usize,
    pub avg_cell_size: f32,
    pub cell_size: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insertion() {
        let mut hash = SpatialHash::new(10.0, 100);
        let entity = hecs::Entity::from_bits(1).unwrap();

        hash.insert(entity, 5.0, 5.0);

        let nearby = hash.get_nearby_entities(5.0, 5.0, 5.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0], entity);
    }

    #[test]
    fn test_spatial_hash_update() {
        let mut hash = SpatialHash::new(10.0, 100);
        let entity = hecs::Entity::from_bits(1).unwrap();

        hash.insert(entity, 5.0, 5.0);
        hash.update_position(entity, 15.0, 15.0);

        let nearby_old = hash.get_nearby_entities(5.0, 5.0, 5.0);
        let nearby_new = hash.get_nearby_entities(15.0, 15.0, 5.0);

        assert_eq!(nearby_old.len(), 0);
        assert_eq!(nearby_new.len(), 1);
        assert_eq!(nearby_new[0], entity);
    }

    #[test]
    fn test_batch_insert() {
        let mut hash = SpatialHash::new(10.0, 100);
        let entity1 = hecs::Entity::from_bits(1).unwrap();
        let entity2 = hecs::Entity::from_bits(2).unwrap();

        let entities = vec![(entity1, 5.0, 5.0), (entity2, 15.0, 15.0)];
        hash.batch_insert(&entities);

        let nearby1 = hash.get_nearby_entities(5.0, 5.0, 5.0);
        let nearby2 = hash.get_nearby_entities(15.0, 15.0, 5.0);

        assert_eq!(nearby1.len(), 1);
        assert_eq!(nearby2.len(), 1);
    }
}
