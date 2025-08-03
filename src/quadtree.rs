use hecs::Entity;

/// Quadtree node for efficient spatial partitioning
#[derive(Debug)]
struct QuadNode {
    bounds: Bounds,
    entities: Vec<(Entity, f32, f32)>, // entity, x, y
    children: Option<[Box<QuadNode>; 4]>,
    max_entities: usize,
    max_depth: usize,
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Bounds {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    fn intersects(&self, other: &Bounds) -> bool {
        !(other.x >= self.x + self.width
            || other.x + other.width <= self.x
            || other.y >= self.y + self.height
            || other.y + other.height <= self.y)
    }

    fn subdivide(&self) -> [Bounds; 4] {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        let mid_x = self.x + half_width;
        let mid_y = self.y + half_height;

        [
            Bounds::new(self.x, self.y, half_width, half_height),           // Top-left
            Bounds::new(mid_x, self.y, half_width, half_height),            // Top-right
            Bounds::new(self.x, mid_y, half_width, half_height),            // Bottom-left
            Bounds::new(mid_x, mid_y, half_width, half_height),             // Bottom-right
        ]
    }
}

impl QuadNode {
    fn new(bounds: Bounds, max_entities: usize, max_depth: usize) -> Self {
        Self {
            bounds,
            entities: Vec::new(),
            children: None,
            max_entities,
            max_depth,
        }
    }

    fn insert(&mut self, entity: Entity, x: f32, y: f32, depth: usize) {
        if !self.bounds.contains(x, y) {
            return;
        }

        if self.children.is_none() && self.entities.len() < self.max_entities {
            self.entities.push((entity, x, y));
            return;
        }

        if self.children.is_none() && depth < self.max_depth {
            self.subdivide();
        }

        if let Some(ref mut children) = self.children {
            let sub_bounds = self.bounds.subdivide();
            for (i, child) in children.iter_mut().enumerate() {
                if sub_bounds[i].contains(x, y) {
                    child.insert(entity, x, y, depth + 1);
                    break;
                }
            }
        } else {
            self.entities.push((entity, x, y));
        }
    }

    fn subdivide(&mut self) {
        let sub_bounds = self.bounds.subdivide();
        let mut children = [
            Box::new(QuadNode::new(sub_bounds[0], self.max_entities, self.max_depth)),
            Box::new(QuadNode::new(sub_bounds[1], self.max_entities, self.max_depth)),
            Box::new(QuadNode::new(sub_bounds[2], self.max_entities, self.max_depth)),
            Box::new(QuadNode::new(sub_bounds[3], self.max_entities, self.max_depth)),
        ];

        // Redistribute existing entities
        for (entity, x, y) in self.entities.drain(..) {
            for (i, child) in children.iter_mut().enumerate() {
                if sub_bounds[i].contains(x, y) {
                    child.insert(entity, x, y, 1);
                    break;
                }
            }
        }

        self.children = Some(children);
    }

    fn query_range(&self, query_bounds: &Bounds, results: &mut Vec<Entity>) {
        if !self.bounds.intersects(query_bounds) {
            return;
        }

        // Add entities from this node
        for (entity, x, y) in &self.entities {
            if query_bounds.contains(*x, *y) {
                results.push(*entity);
            }
        }

        // Recursively query children
        if let Some(ref children) = self.children {
            for child in children {
                child.query_range(query_bounds, results);
            }
        }
    }

    fn query_radius(&self, center_x: f32, center_y: f32, radius: f32, results: &mut Vec<Entity>) {
        let radius_sq = radius * radius;
        
        // Check if this node's bounds intersect with the query circle
        let closest_x = (self.bounds.x).max(center_x - radius).min(center_x + radius);
        let closest_y = (self.bounds.y).max(center_y - radius).min(center_y + radius);
        
        let distance_sq = (closest_x - center_x).powi(2) + (closest_y - center_y).powi(2);
        
        if distance_sq > radius_sq + self.bounds.width * self.bounds.height {
            return;
        }

        // Add entities from this node
        for (entity, x, y) in &self.entities {
            let distance_sq = (x - center_x).powi(2) + (y - center_y).powi(2);
            if distance_sq <= radius_sq {
                results.push(*entity);
            }
        }

        // Recursively query children
        if let Some(ref children) = self.children {
            for child in children {
                child.query_radius(center_x, center_y, radius, results);
            }
        }
    }

    fn clear(&mut self) {
        self.entities.clear();
        self.children = None;
    }
}

/// High-performance spatial data structure for large numbers of entities
pub struct Quadtree {
    root: QuadNode,
    world_bounds: Bounds,
}

impl Quadtree {
    pub fn new(world_size: f32, max_entities_per_node: usize, max_depth: usize) -> Self {
        let world_bounds = Bounds::new(-world_size / 2.0, -world_size / 2.0, world_size, world_size);
        let root = QuadNode::new(world_bounds.clone(), max_entities_per_node, max_depth);
        
        Self {
            root,
            world_bounds,
        }
    }

    pub fn clear(&mut self) {
        self.root.clear();
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        self.root.insert(entity, x, y, 0);
    }

    pub fn query_range(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<Entity> {
        let query_bounds = Bounds::new(x - width / 2.0, y - height / 2.0, width, height);
        let mut results = Vec::new();
        self.root.query_range(&query_bounds, &mut results);
        results
    }

    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        self.root.query_radius(x, y, radius, &mut results);
        results
    }

    pub fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        self.query_radius(x, y, radius)
    }

    /// Optimized version that pre-allocates the result vector
    pub fn get_nearby_entities_optimized(&self, x: f32, y: f32, radius: f32, limit: usize) -> Vec<Entity> {
        let mut results = Vec::with_capacity(limit);
        self.root.query_radius(x, y, radius, &mut results);
        
        // Limit results and shuffle to avoid bias
        if results.len() > limit {
            results.truncate(limit);
            use rand::seq::SliceRandom;
            use rand::thread_rng;
            results.shuffle(&mut thread_rng());
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree_insertion() {
        let mut quadtree = Quadtree::new(1000.0, 10, 8);
        let entity = hecs::Entity::from_bits(1).unwrap();
        
        quadtree.insert(entity, 0.0, 0.0);
        
        let nearby = quadtree.get_nearby_entities(0.0, 0.0, 10.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0], entity);
    }

    #[test]
    fn test_quadtree_query_radius() {
        let mut quadtree = Quadtree::new(1000.0, 10, 8);
        let entity1 = hecs::Entity::from_bits(1).unwrap();
        let entity2 = hecs::Entity::from_bits(2).unwrap();
        
        quadtree.insert(entity1, 0.0, 0.0);
        quadtree.insert(entity2, 100.0, 100.0);
        
        let nearby = quadtree.get_nearby_entities(0.0, 0.0, 5.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0], entity1);
    }

    #[test]
    fn test_quadtree_clear() {
        let mut quadtree = Quadtree::new(1000.0, 10, 8);
        let entity = hecs::Entity::from_bits(1).unwrap();
        
        quadtree.insert(entity, 0.0, 0.0);
        quadtree.clear();
        
        let nearby = quadtree.get_nearby_entities(0.0, 0.0, 10.0);
        assert_eq!(nearby.len(), 0);
    }
} 