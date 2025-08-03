use crate::components::Color;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

// Grouped gene structures for better organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementGenes {
    pub speed: f32,
    pub sense_radius: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnergyGenes {
    pub efficiency: f32,
    pub loss_rate: f32,
    pub gain_rate: f32,
    pub size_factor: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReproductionGenes {
    pub rate: f32,
    pub mutation_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppearanceGenes {
    pub hue: f32,
    pub saturation: f32,
}

// Main genes structure that groups related traits
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub movement: MovementGenes,
    pub energy: EnergyGenes,
    pub reproduction: ReproductionGenes,
    pub appearance: AppearanceGenes,
}

impl Genes {
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        Self {
            movement: MovementGenes {
                speed: rng.gen_range(0.2..1.5),
                sense_radius: rng.gen_range(10.0..100.0),
            },
            energy: EnergyGenes {
                efficiency: rng.gen_range(0.5..2.0),
                loss_rate: rng.gen_range(0.1..1.0),
                gain_rate: rng.gen_range(0.5..3.0),
                size_factor: rng.gen_range(0.5..2.0),
            },
            reproduction: ReproductionGenes {
                rate: rng.gen_range(0.001..0.1),
                mutation_rate: rng.gen_range(0.01..0.1),
            },
            appearance: AppearanceGenes {
                hue: rng.gen_range(0.0..1.0),
                saturation: rng.gen_range(0.3..1.0),
            },
        }
    }

    pub fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Movement mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.movement.speed =
                (new_genes.movement.speed + rng.gen_range(-0.1..0.1)).clamp(0.1, 2.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.movement.sense_radius =
                (new_genes.movement.sense_radius + rng.gen_range(-5.0..5.0)).clamp(5.0, 120.0);
        }

        // Energy mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.efficiency =
                (new_genes.energy.efficiency + rng.gen_range(-0.1..0.1)).clamp(0.3, 3.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.loss_rate =
                (new_genes.energy.loss_rate + rng.gen_range(-0.1..0.1)).clamp(0.05, 2.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.gain_rate =
                (new_genes.energy.gain_rate + rng.gen_range(-0.2..0.2)).clamp(0.2, 4.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.size_factor =
                (new_genes.energy.size_factor + rng.gen_range(-0.1..0.1)).clamp(0.2, 3.0);
        }

        // Reproduction mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.reproduction.rate =
                (new_genes.reproduction.rate + rng.gen_range(-0.02..0.02)).clamp(0.0001, 0.2);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.reproduction.mutation_rate = (new_genes.reproduction.mutation_rate
                + rng.gen_range(-0.02..0.02))
            .clamp(0.001, 0.2);
        }

        // Appearance mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.appearance.hue =
                (new_genes.appearance.hue + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.appearance.saturation =
                (new_genes.appearance.saturation + rng.gen_range(-0.1..0.1)).clamp(0.1, 1.0);
        }

        new_genes
    }

    pub fn get_color(&self) -> Color {
        Color::from_hsv(self.appearance.hue, self.appearance.saturation, 0.8)
    }

    // Predation logic based on genes
    pub fn can_eat(
        &self,
        other_genes: &Genes,
        other_size: &crate::components::Size,
        self_size: &crate::components::Size,
    ) -> bool {
        let size_advantage = self_size.radius / other_size.radius;
        let speed_advantage = self.movement.speed / other_genes.movement.speed;

        // Need significant size and speed advantage to be a successful predator
        size_advantage > 1.2 && speed_advantage > 0.8
    }

    // Energy gain calculation
    pub fn get_energy_gain(
        &self,
        other_energy: f32,
        other_size: &crate::components::Size,
        self_size: &crate::components::Size,
    ) -> f32 {
        let size_ratio = other_size.radius / self_size.radius;
        let base_gain = other_energy * self.energy.gain_rate * 0.3;

        // Bigger prey = more energy, but with stronger diminishing returns
        base_gain * (1.0 + size_ratio * 0.3).min(1.5)
    }

    // Convenience getters for backward compatibility
    pub fn speed(&self) -> f32 {
        self.movement.speed
    }
    pub fn sense_radius(&self) -> f32 {
        self.movement.sense_radius
    }
    pub fn energy_efficiency(&self) -> f32 {
        self.energy.efficiency
    }
    pub fn reproduction_rate(&self) -> f32 {
        self.reproduction.rate
    }
    pub fn mutation_rate(&self) -> f32 {
        self.reproduction.mutation_rate
    }
    pub fn size_factor(&self) -> f32 {
        self.energy.size_factor
    }
    pub fn energy_loss_rate(&self) -> f32 {
        self.energy.loss_rate
    }
    pub fn energy_gain_rate(&self) -> f32 {
        self.energy.gain_rate
    }
    pub fn color_hue(&self) -> f32 {
        self.appearance.hue
    }
    pub fn color_saturation(&self) -> f32 {
        self.appearance.saturation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Size;

    #[test]
    fn test_genes_new_random() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);

        // Test movement genes
        assert!(genes.movement.speed >= 0.2 && genes.movement.speed <= 1.5);
        assert!(genes.movement.sense_radius >= 10.0 && genes.movement.sense_radius <= 100.0);

        // Test energy genes
        assert!(genes.energy.efficiency >= 0.5 && genes.energy.efficiency <= 2.0);
        assert!(genes.energy.loss_rate >= 0.1 && genes.energy.loss_rate <= 1.0);
        assert!(genes.energy.gain_rate >= 0.5 && genes.energy.gain_rate <= 3.0);
        assert!(genes.energy.size_factor >= 0.5 && genes.energy.size_factor <= 2.0);

        // Test reproduction genes
        assert!(genes.reproduction.rate >= 0.001 && genes.reproduction.rate <= 0.1);
        assert!(
            genes.reproduction.mutation_rate >= 0.01 && genes.reproduction.mutation_rate <= 0.1
        );

        // Test appearance genes
        assert!(genes.appearance.hue >= 0.0 && genes.appearance.hue <= 1.0);
        assert!(genes.appearance.saturation >= 0.3 && genes.appearance.saturation <= 1.0);
    }

    #[test]
    fn test_genes_mutation() {
        let mut rng = thread_rng();
        let original_genes = Genes::new_random(&mut rng);
        let mutated_genes = original_genes.mutate(&mut rng);

        // Test that genes are within valid ranges after mutation
        assert!(mutated_genes.movement.speed >= 0.1 && mutated_genes.movement.speed <= 2.0);
        assert!(
            mutated_genes.movement.sense_radius >= 5.0
                && mutated_genes.movement.sense_radius <= 120.0
        );
        assert!(mutated_genes.energy.efficiency >= 0.3 && mutated_genes.energy.efficiency <= 3.0);
        assert!(mutated_genes.energy.loss_rate >= 0.05 && mutated_genes.energy.loss_rate <= 2.0);
        assert!(mutated_genes.energy.gain_rate >= 0.2 && mutated_genes.energy.gain_rate <= 4.0);
        assert!(mutated_genes.energy.size_factor >= 0.2 && mutated_genes.energy.size_factor <= 3.0);
        assert!(
            mutated_genes.reproduction.rate >= 0.0001 && mutated_genes.reproduction.rate <= 0.2
        );
        assert!(
            mutated_genes.reproduction.mutation_rate >= 0.005
                && mutated_genes.reproduction.mutation_rate <= 0.15
        );
        assert!(mutated_genes.appearance.hue >= 0.0 && mutated_genes.appearance.hue <= 1.0);
        assert!(
            mutated_genes.appearance.saturation >= 0.2
                && mutated_genes.appearance.saturation <= 1.1
        );
    }

    #[test]
    fn test_genes_get_color() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let color = genes.get_color();

        // Test that color values are in valid range
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
    }

    #[test]
    fn test_genes_can_eat() {
        let mut rng = thread_rng();
        let predator_genes = Genes::new_random(&mut rng);
        let prey_genes = Genes::new_random(&mut rng);

        let large_predator = Size { radius: 15.0 };
        let small_prey = Size { radius: 5.0 };
        let large_prey = Size { radius: 20.0 };

        // Large predator should be able to eat small prey
        let size_advantage = large_predator.radius / small_prey.radius;
        let speed_advantage = predator_genes.movement.speed / prey_genes.movement.speed;

        if size_advantage > 1.2 && speed_advantage > 0.8 {
            assert!(predator_genes.can_eat(&prey_genes, &small_prey, &large_predator));
        }

        // Small entity should not be able to eat large entity
        assert!(!prey_genes.can_eat(&predator_genes, &large_prey, &small_prey));
    }

    #[test]
    fn test_genes_get_energy_gain() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let other_size = Size { radius: 10.0 };
        let self_size = Size { radius: 8.0 };

        let energy_gain = genes.get_energy_gain(50.0, &other_size, &self_size);

        // Energy gain should be positive and reasonable
        assert!(energy_gain > 0.0);
        assert!(energy_gain <= 50.0); // Should not exceed the original energy
    }

    #[test]
    fn test_genes_getter_methods() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);

        // Test all getter methods
        assert_eq!(genes.speed(), genes.movement.speed);
        assert_eq!(genes.sense_radius(), genes.movement.sense_radius);
        assert_eq!(genes.energy_efficiency(), genes.energy.efficiency);
        assert_eq!(genes.reproduction_rate(), genes.reproduction.rate);
        assert_eq!(genes.mutation_rate(), genes.reproduction.mutation_rate);
        assert_eq!(genes.size_factor(), genes.energy.size_factor);
        assert_eq!(genes.energy_loss_rate(), genes.energy.loss_rate);
        assert_eq!(genes.energy_gain_rate(), genes.energy.gain_rate);
        assert_eq!(genes.color_hue(), genes.appearance.hue);
        assert_eq!(genes.color_saturation(), genes.appearance.saturation);
    }

    #[test]
    fn test_genes_serialization() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);

        // Test serialization and deserialization
        let serialized = serde_json::to_string(&genes).unwrap();
        let deserialized: Genes = serde_json::from_str(&serialized).unwrap();

        assert_eq!(genes.movement.speed, deserialized.movement.speed);
        assert_eq!(
            genes.movement.sense_radius,
            deserialized.movement.sense_radius
        );
        assert_eq!(genes.energy.efficiency, deserialized.energy.efficiency);
        assert_eq!(genes.reproduction.rate, deserialized.reproduction.rate);
        assert_eq!(genes.appearance.hue, deserialized.appearance.hue);
    }

    #[test]
    fn test_genes_clone() {
        let mut rng = thread_rng();
        let original = Genes::new_random(&mut rng);
        let cloned = original.clone();

        assert_eq!(original.movement.speed, cloned.movement.speed);
        assert_eq!(original.movement.sense_radius, cloned.movement.sense_radius);
        assert_eq!(original.energy.efficiency, cloned.energy.efficiency);
        assert_eq!(original.reproduction.rate, cloned.reproduction.rate);
        assert_eq!(original.appearance.hue, cloned.appearance.hue);
    }
}
