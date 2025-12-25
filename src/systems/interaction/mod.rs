use crate::components::{Energy, Position, Size};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::{Entity, World};

pub struct InteractionSystem;

pub struct InteractionParams<'a> {
    pub new_energy: &'a mut f32,
    pub eaten_entity: &'a mut Option<Entity>,
    pub new_pos: &'a Position,
    pub size: &'a Size,
    pub genes: &'a Genes,
    pub nearby_entities: &'a [Entity],
    pub world: &'a World,
    pub config: &'a SimulationConfig,
}

impl InteractionSystem {
    pub fn handle_interactions(&self, params: InteractionParams) {
        let InteractionParams {
            new_energy,
            eaten_entity,
            new_pos,
            size,
            genes,
            nearby_entities,
            world,
            config,
        } = params;
        for &entity in nearby_entities {
            if self.can_interact_with_entity(entity, new_pos, size, genes, world, config) {
                self.process_interaction(entity, new_energy, eaten_entity, genes, world);
                break; // Only interact with one entity per frame
            }
        }
    }

    fn can_interact_with_entity(
        &self,
        entity: Entity,
        new_pos: &Position,
        size: &Size,
        genes: &Genes,
        world: &World,
        config: &SimulationConfig,
    ) -> bool {
        if let Ok(nearby_pos) = world.get::<&Position>(entity) {
            if let Ok(nearby_genes) = world.get::<&Genes>(entity) {
                if let Ok(nearby_energy) = world.get::<&Energy>(entity) {
                    if let Ok(nearby_size) = world.get::<&Size>(entity) {
                        if nearby_energy.current > 0.0 {
                            let distance = self.calculate_distance(new_pos, &nearby_pos);
                            if distance < (size.radius + config.physics.interaction_radius_offset) {
                                return genes.can_eat(&nearby_genes, &nearby_size, size);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn calculate_distance(&self, pos1: &Position, pos2: &Position) -> f32 {
        ((pos2.x - pos1.x).powi(2) + (pos2.y - pos1.y).powi(2)).sqrt()
    }

    fn process_interaction(
        &self,
        entity: Entity,
        new_energy: &mut f32,
        eaten_entity: &mut Option<Entity>,
        genes: &Genes,
        world: &World,
    ) {
        if let Ok(nearby_energy) = world.get::<&Energy>(entity) {
            if let Ok(nearby_size) = world.get::<&Size>(entity) {
                if let Ok(nearby_genes) = world.get::<&Genes>(entity) {
                    *eaten_entity = Some(entity);
                    let energy_gained = genes.get_energy_gain(
                        nearby_energy.current,
                        &nearby_size,
                        &Size { radius: 1.0 },
                        &nearby_genes,
                    );
                    *new_energy =
                        (*new_energy + energy_gained - 0.5).min(genes.energy_efficiency() * 100.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
