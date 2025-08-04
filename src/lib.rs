use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

mod components;
mod config;
mod genes;
mod simulation;
mod spatial_grid;
mod stats;
mod systems;

#[cfg(target_arch = "wasm32")]
mod web;

// Re-export the thread pool initialization
pub use wasm_bindgen_rayon::init_thread_pool;

#[derive(Serialize, Deserialize)]
struct EntityData {
    x: f32,
    y: f32,
    radius: f32,
    r: f32,
    g: f32,
    b: f32,
}

#[wasm_bindgen]
pub struct WebSimulation {
    simulation: simulation::Simulation,
    config: config::SimulationConfig,
}

#[wasm_bindgen]
impl WebSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(world_size: f32, config_json: &str) -> Result<WebSimulation, JsValue> {
        let config: config::SimulationConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;

        let simulation = simulation::Simulation::new_with_config(world_size, config.clone());

        Ok(WebSimulation { simulation, config })
    }

    pub fn update(&mut self) {
        self.simulation.update();
    }

    pub fn get_entities(&self) -> JsValue {
        let entity_tuples = self.simulation.get_entities();
        let entities: Vec<EntityData> = entity_tuples
            .into_iter()
            .map(|(x, y, radius, r, g, b)| EntityData { x, y, radius, r, g, b })
            .collect();
        serde_wasm_bindgen::to_value(&entities).unwrap_or(JsValue::NULL)
    }

    pub fn get_stats(&self) -> JsValue {
        let stats = stats::SimulationStats::from_world(
            self.simulation.world(),
            self.config.population.max_population as f32,
            self.config.population.entity_scale,
        );
        serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
    }

    pub fn get_world_size(&self) -> f32 {
        self.simulation.world_size()
    }

    pub fn get_step(&self) -> u32 {
        self.simulation.step()
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
