use wasm_bindgen::prelude::*;

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
        let entities = self.simulation.get_entities();
        serde_wasm_bindgen::to_value(&entities).unwrap_or_else(|_| JsValue::NULL)
    }

    pub fn get_stats(&self) -> JsValue {
        let stats = stats::SimulationStats::from_world(
            self.simulation.world(),
            self.config.max_population as f32,
            self.config.entity_scale,
        );
        serde_wasm_bindgen::to_value(&stats).unwrap_or_else(|_| JsValue::NULL)
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
