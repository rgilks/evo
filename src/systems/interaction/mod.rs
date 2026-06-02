use crate::components::{Position, Size};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::Entity;

pub struct InteractionSystem;

impl crate::systems::System for InteractionSystem {
    fn run(&self, ctx: &mut crate::systems::EntityContext) {
        let eaten = self.handle_interactions(InteractionParams {
            new_energy: &mut ctx.new_energy,
            new_pos: &ctx.new_pos,
            size: ctx.size,
            genes: ctx.genes,
            nearby_entities: ctx.nearby_entities,
            cache: ctx.cache,
            config: ctx.config,
        });
        ctx.eaten_entity = eaten;
    }
}

pub struct InteractionParams<'a> {
    pub new_energy: &'a mut f32,
    pub new_pos: &'a Position,
    pub size: &'a Size,
    pub genes: &'a Genes,
    pub nearby_entities: &'a [Entity],
    pub cache: &'a crate::systems::NeighborCache,
    pub config: &'a SimulationConfig,
}

impl InteractionSystem {
    pub fn handle_interactions(&self, params: InteractionParams) -> Option<Entity> {
        let InteractionParams {
            new_energy,
            new_pos,
            size,
            genes,
            nearby_entities,
            cache,
            config,
        } = params;
        for &entity in nearby_entities {
            if self.can_interact_with_entity(entity, new_pos, size, genes, cache, config) {
                self.process_interaction(entity, new_energy, genes, cache);
                return Some(entity); // Eat one entity per frame
            }
        }
        None
    }

    fn can_interact_with_entity(
        &self,
        entity: Entity,
        new_pos: &Position,
        size: &Size,
        genes: &Genes,
        cache: &crate::systems::NeighborCache,
        config: &SimulationConfig,
    ) -> bool {
        if let Some(n) = cache.get(&entity) {
            if n.energy.current > 0.0 {
                let distance = self.calculate_distance(new_pos, &n.pos);
                if distance < (size.radius + config.physics.interaction_radius_offset) {
                    return genes.can_eat(&n.genes, &n.size, size);
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
        genes: &Genes,
        cache: &crate::systems::NeighborCache,
    ) {
        if let Some(n) = cache.get(&entity) {
            let energy_gained =
                genes.get_energy_gain(n.energy.current, &n.size, &Size { radius: 1.0 }, &n.genes);
            *new_energy =
                (*new_energy + energy_gained - 0.5).min(genes.energy_efficiency() * 100.0);
        }
    }
}

#[cfg(test)]
mod tests;
