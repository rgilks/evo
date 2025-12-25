use crate::components::{Color, MovementStyle, MovementType};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorGenes {
    pub movement_style: MovementStyle,
    pub gene_preference_strength: f32, // How strongly to prefer different genes (0.0 = no preference, 1.0 = strong preference)
    pub social_tendency: f32, // Tendency to be social vs solitary (0.0 = solitary, 1.0 = social)
}

// Main genes structure that groups related traits
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub movement: MovementGenes,
    pub energy: EnergyGenes,
    pub reproduction: ReproductionGenes,
    pub appearance: AppearanceGenes,
    pub behavior: BehaviorGenes,
}

impl Genes {
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        let movement_type = match rng.gen_range(0..5) {
            0 => MovementType::Random,
            1 => MovementType::Flocking,
            2 => MovementType::Solitary,
            3 => MovementType::Predatory,
            _ => MovementType::Grazing,
        };

        Self {
            movement: MovementGenes {
                speed: rng.gen_range(0.1..2.5),
                sense_radius: rng.gen_range(5.0..150.0),
            },
            energy: EnergyGenes {
                efficiency: rng.gen_range(0.3..3.0),
                loss_rate: rng.gen_range(0.05..2.0),
                gain_rate: rng.gen_range(0.2..4.5),
                size_factor: rng.gen_range(0.3..2.5),
            },
            reproduction: ReproductionGenes {
                rate: rng.gen_range(0.0005..0.15),
                mutation_rate: rng.gen_range(0.005..0.15),
            },
            appearance: AppearanceGenes {
                hue: rng.gen_range(0.0..1.0),
                saturation: rng.gen_range(0.2..1.0),
            },
            behavior: BehaviorGenes {
                movement_style: MovementStyle {
                    style: movement_type,
                    flocking_strength: rng.gen_range(0.0..1.0),
                    separation_distance: rng.gen_range(5.0..25.0),
                    alignment_strength: rng.gen_range(0.0..1.0),
                    cohesion_strength: rng.gen_range(0.0..1.0),
                },
                gene_preference_strength: rng.gen_range(0.0..1.0),
                social_tendency: rng.gen_range(0.0..1.0),
            },
        }
    }

    pub fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Movement mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.movement.speed =
                (new_genes.movement.speed + rng.gen_range(-0.15..0.15)).clamp(0.05, 3.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.movement.sense_radius =
                (new_genes.movement.sense_radius + rng.gen_range(-8.0..8.0)).clamp(2.0, 180.0);
        }

        // Energy mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.efficiency =
                (new_genes.energy.efficiency + rng.gen_range(-0.15..0.15)).clamp(0.2, 4.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.loss_rate =
                (new_genes.energy.loss_rate + rng.gen_range(-0.15..0.15)).clamp(0.02, 3.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.gain_rate =
                (new_genes.energy.gain_rate + rng.gen_range(-0.25..0.25)).clamp(0.1, 5.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.energy.size_factor =
                (new_genes.energy.size_factor + rng.gen_range(-0.15..0.15)).clamp(0.1, 3.5);
        }

        // Reproduction mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.reproduction.rate =
                (new_genes.reproduction.rate + rng.gen_range(-0.025..0.025)).clamp(0.0001, 0.25);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.reproduction.mutation_rate = (new_genes.reproduction.mutation_rate
                + rng.gen_range(-0.025..0.025))
            .clamp(0.001, 0.25);
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

        // Behavior mutations
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.movement_style.flocking_strength =
                (new_genes.behavior.movement_style.flocking_strength + rng.gen_range(-0.1..0.1))
                    .clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.movement_style.separation_distance =
                (new_genes.behavior.movement_style.separation_distance + rng.gen_range(-2.0..2.0))
                    .clamp(2.0, 30.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.movement_style.alignment_strength =
                (new_genes.behavior.movement_style.alignment_strength + rng.gen_range(-0.1..0.1))
                    .clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.movement_style.cohesion_strength =
                (new_genes.behavior.movement_style.cohesion_strength + rng.gen_range(-0.1..0.1))
                    .clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.gene_preference_strength =
                (new_genes.behavior.gene_preference_strength + rng.gen_range(-0.1..0.1))
                    .clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.reproduction.mutation_rate {
            new_genes.behavior.social_tendency =
                (new_genes.behavior.social_tendency + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }

        // Occasionally change movement type
        if rng.gen::<f32>() < self.reproduction.mutation_rate * 0.1 {
            new_genes.behavior.movement_style.style = match rng.gen_range(0..5) {
                0 => MovementType::Random,
                1 => MovementType::Flocking,
                2 => MovementType::Solitary,
                3 => MovementType::Predatory,
                _ => MovementType::Grazing,
            };
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

    // Calculate how similar two sets of genes are (0.0 = identical, 1.0 = completely different)
    pub fn calculate_gene_similarity(&self, other: &Genes) -> f32 {
        let mut total_difference = 0.0;
        let mut total_weights = 0.0;

        // Movement genes similarity
        let speed_diff = (self.movement.speed - other.movement.speed).abs() / 2.5; // Normalize by max speed
        let sense_diff = (self.movement.sense_radius - other.movement.sense_radius).abs() / 180.0; // Normalize by max sense radius
        total_difference += speed_diff * 0.3 + sense_diff * 0.2;
        total_weights += 0.5;

        // Energy genes similarity
        let efficiency_diff = (self.energy.efficiency - other.energy.efficiency).abs() / 4.0;
        let loss_diff = (self.energy.loss_rate - other.energy.loss_rate).abs() / 3.0;
        let gain_diff = (self.energy.gain_rate - other.energy.gain_rate).abs() / 5.0;
        let size_factor_diff = (self.energy.size_factor - other.energy.size_factor).abs() / 3.5;
        total_difference +=
            efficiency_diff * 0.15 + loss_diff * 0.15 + gain_diff * 0.1 + size_factor_diff * 0.1;
        total_weights += 0.5;

        // Appearance genes similarity (color-based)
        let hue_diff = (self.appearance.hue - other.appearance.hue).abs();
        let saturation_diff = (self.appearance.saturation - other.appearance.saturation).abs();
        total_difference += hue_diff * 0.3 + saturation_diff * 0.2;
        total_weights += 0.5;

        // Behavior genes similarity
        let flocking_diff = (self.behavior.movement_style.flocking_strength
            - other.behavior.movement_style.flocking_strength)
            .abs();
        let social_diff = (self.behavior.social_tendency - other.behavior.social_tendency).abs();
        let preference_diff = (self.behavior.gene_preference_strength
            - other.behavior.gene_preference_strength)
            .abs();
        total_difference += flocking_diff * 0.2 + social_diff * 0.2 + preference_diff * 0.1;
        total_weights += 0.5;

        // Movement type similarity
        let type_similarity = if std::mem::discriminant(&self.behavior.movement_style.style)
            == std::mem::discriminant(&other.behavior.movement_style.style)
        {
            0.0 // Same type
        } else {
            1.0 // Different type
        };
        total_difference += type_similarity * 0.3;
        total_weights += 0.3;

        total_difference / total_weights
    }

    // Get predation preference based on gene similarity
    pub fn get_predation_preference(&self, other_genes: &Genes) -> f32 {
        let gene_similarity = self.calculate_gene_similarity(other_genes);

        // Higher preference for different genes (inverse of similarity)
        // Apply the gene preference strength to modulate this effect
        let base_preference = 1.0 - gene_similarity;
        let modulated_preference = base_preference * self.behavior.gene_preference_strength;

        // Add a small base preference so entities can still eat similar ones if needed
        modulated_preference + (1.0 - self.behavior.gene_preference_strength) * 0.3
    }

    // Energy gain calculation
    pub fn get_energy_gain(
        &self,
        other_energy: f32,
        other_size: &crate::components::Size,
        self_size: &crate::components::Size,
        other_genes: &Genes,
    ) -> f32 {
        let size_ratio = other_size.radius / self_size.radius;
        let base_gain = other_energy * self.energy.gain_rate * 0.3;

        // Bigger prey = more energy, but with stronger diminishing returns
        let size_bonus = base_gain * (1.0 + size_ratio * 0.3).min(1.5);

        // Gene preference bonus - more energy from preferred prey
        let gene_bonus = self.get_predation_preference(other_genes);

        size_bonus * (1.0 + gene_bonus * 0.5) // Up to 50% bonus for preferred prey
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

    pub fn size_factor(&self) -> f32 {
        self.energy.size_factor
    }
    pub fn energy_loss_rate(&self) -> f32 {
        self.energy.loss_rate
    }
}

#[cfg(test)]
mod tests;
