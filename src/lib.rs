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
    entity_buffer: Vec<f32>, // Reusable buffer for entity data
}

#[wasm_bindgen]
impl WebSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(world_size: f32, config_json: &str) -> Result<WebSimulation, JsValue> {
        let config: config::SimulationConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;

        let simulation = simulation::Simulation::new_with_config(world_size, config.clone());

        Ok(WebSimulation {
            simulation,
            config,
            entity_buffer: Vec::with_capacity(60000), // 10000 entities * 6 floats
        })
    }

    pub fn update(&mut self) {
        self.simulation.update();
    }

    /// Update entity buffer and return pointer for WebGPU renderer
    pub fn update_entity_buffer(&mut self) -> *const f32 {
        let entity_tuples = self.simulation.get_entities();
        self.entity_buffer.clear();

        for (x, y, radius, r, g, b) in entity_tuples {
            self.entity_buffer.push(x);
            self.entity_buffer.push(y);
            self.entity_buffer.push(radius);
            self.entity_buffer.push(r);
            self.entity_buffer.push(g);
            self.entity_buffer.push(b);
        }

        self.entity_buffer.as_ptr()
    }

    pub fn entity_count(&self) -> u32 {
        (self.entity_buffer.len() / 6) as u32
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

    pub fn update_param(&mut self, name: &str, value: f32) {
        match name {
            "max_velocity" => self.config.physics.max_velocity = value,
            "center_pressure" => self.config.physics.center_pressure_strength = value,
            "death_chance" => self.config.reproduction.death_chance_factor = value,
            "repro_threshold" => self.config.reproduction.reproduction_energy_threshold = value,
            "energy_cost" => self.config.energy.size_energy_cost_factor = value,
            "bounce_factor" => self.config.physics.velocity_bounce_factor = value,
            _ => {}
        }
        self.simulation.update_config(self.config.clone());
    }

    pub fn get_step(&self) -> u32 {
        self.simulation.step()
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
