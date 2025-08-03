use hecs::Entity;
use rayon::prelude::*;

/// Batch processor for efficient handling of large numbers of entities
pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    /// Process entities in batches for better cache locality and parallelization
    pub fn process_entities_batched<T, F, R>(
        &self,
        entities: &[T],
        processor: F,
    ) -> Vec<R>
    where
        T: Sync,
        F: Fn(&[T]) -> Vec<R> + Send + Sync,
        R: Send,
    {
        entities
            .par_chunks(self.batch_size)
            .flat_map(|chunk| processor(chunk))
            .collect()
    }

    /// Optimized distance calculation using SIMD-friendly operations
    pub fn calculate_distances_squared(
        &self,
        positions: &[(Entity, f32, f32)],
        center_x: f32,
        center_y: f32,
    ) -> Vec<(Entity, f32)> {
        positions
            .par_iter()
            .map(|(entity, x, y)| {
                let dx = x - center_x;
                let dy = y - center_y;
                let distance_sq = dx * dx + dy * dy;
                (*entity, distance_sq)
            })
            .collect()
    }

    /// Batch entity updates with pre-allocated vectors
    pub fn batch_entity_updates<T: Clone>(
        &self,
        updates: Vec<T>,
    ) -> Vec<Vec<T>> {
        updates
            .into_iter()
            .collect::<Vec<_>>()
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Optimized spatial queries with result limiting
    pub fn spatial_query_batched(
        &self,
        entities: &[(Entity, f32, f32)],
        query_x: f32,
        query_y: f32,
        radius: f32,
        limit: usize,
    ) -> Vec<Entity> {
        let radius_sq = radius * radius;
        
        let mut candidates: Vec<(Entity, f32)> = entities
            .par_iter()
            .filter_map(|(entity, x, y)| {
                let dx = x - query_x;
                let dy = y - query_y;
                let distance_sq = dx * dx + dy * dy;
                
                if distance_sq <= radius_sq {
                    Some((*entity, distance_sq))
                } else {
                    None
                }
            })
            .collect();

        // Sort by distance and limit results
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates
            .into_iter()
            .take(limit)
            .map(|(entity, _)| entity)
            .collect()
    }

    /// Memory-efficient entity processing with object pooling
    pub fn process_with_pool<T, F, R>(
        &self,
        entities: &[T],
        processor: F,
        pool_size: usize,
    ) -> Vec<R>
    where
        T: Sync,
        F: Fn(&T) -> R + Send + Sync,
        R: Send,
    {
        // Use rayon's built-in work-stealing for optimal parallelization
        entities
            .par_iter()
            .with_min_len(pool_size)
            .map(processor)
            .collect()
    }
}

/// SIMD-optimized mathematical operations
pub mod simd_math {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    /// Vectorized distance calculation (when SIMD is available)
    #[cfg(target_arch = "x86_64")]
    pub fn distance_squared_simd(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        unsafe {
            let dx = _mm_sub_ps(
                _mm_set_ps(x1, x1, x1, x1),
                _mm_set_ps(x2, x2, x2, x2)
            );
            let dy = _mm_sub_ps(
                _mm_set_ps(y1, y1, y1, y1),
                _mm_set_ps(y2, y2, y2, y2)
            );
            
            let dx_sq = _mm_mul_ps(dx, dx);
            let dy_sq = _mm_mul_ps(dy, dy);
            let sum = _mm_add_ps(dx_sq, dy_sq);
            
            let mut result = [0.0f32; 4];
            _mm_store_ps(result.as_mut_ptr(), sum);
            result[0]
        }
    }

    /// Fallback for non-SIMD architectures
    #[cfg(not(target_arch = "x86_64"))]
    pub fn distance_squared_simd(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        let dx = x1 - x2;
        let dy = y1 - y2;
        dx * dx + dy * dy
    }

    /// Vectorized batch distance calculations
    pub fn batch_distances_squared(
        positions: &[(f32, f32)],
        center: (f32, f32),
    ) -> Vec<f32> {
        positions
            .iter()
            .map(|(x, y)| distance_squared_simd(*x, *y, center.0, center.1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(100);
        let entities = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        let result = processor.process_entities_batched(&entities, |chunk| {
            chunk.iter().map(|&x| x * 2).collect::<Vec<_>>()
        });
        
        assert_eq!(result.len(), 10);
        assert_eq!(result[0], 2);
    }

    #[test]
    fn test_distance_calculation() {
        let processor = BatchProcessor::new(100);
        let positions = vec![
            (hecs::Entity::from_bits(1).unwrap(), 0.0, 0.0),
            (hecs::Entity::from_bits(2).unwrap(), 3.0, 4.0),
            (hecs::Entity::from_bits(3).unwrap(), 5.0, 12.0),
        ];
        
        let distances = processor.calculate_distances_squared(&positions, 0.0, 0.0);
        
        assert_eq!(distances.len(), 3);
        assert_eq!(distances[0].1, 0.0); // Distance to (0,0)
        assert_eq!(distances[1].1, 25.0); // Distance to (3,4) = 5²
        assert_eq!(distances[2].1, 169.0); // Distance to (5,12) = 13²
    }

    #[test]
    fn test_simd_math() {
        let distance = simd_math::distance_squared_simd(0.0, 0.0, 3.0, 4.0);
        assert_eq!(distance, 25.0);
    }
} 