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

        // Seed from the wall clock so each page load is a different run, while the
        // run stays fully reproducible from this seed (logged for sharing/replay).
        let seed = js_sys::Date::now() as u64;
        web_sys::console::log_1(&JsValue::from_str(&format!("Simulation seed: {seed}")));
        let simulation =
            simulation::Simulation::new_with_config_seeded(world_size, config.clone(), seed);

        Ok(WebSimulation {
            simulation,
            config,
            entity_buffer: Vec::with_capacity(80000), // 10000 entities * 8 floats
        })
    }

    pub fn update(&mut self) {
        self.simulation.update();
    }

    /// Update entity buffer and return pointer for WebGPU renderer
    pub fn update_entity_buffer(&mut self) -> *const f32 {
        let entity_tuples = self.simulation.get_entities();
        self.entity_buffer.clear();

        for (px, py, cx, cy, radius, r, g, b) in entity_tuples {
            self.entity_buffer.push(px);
            self.entity_buffer.push(py);
            self.entity_buffer.push(cx);
            self.entity_buffer.push(cy);
            self.entity_buffer.push(radius);
            self.entity_buffer.push(r);
            self.entity_buffer.push(g);
            self.entity_buffer.push(b);
        }

        self.entity_buffer.as_ptr()
    }

    pub fn entity_count(&self) -> u32 {
        (self.entity_buffer.len() / 8) as u32
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

    pub fn update_param(&mut self, param: SimParam, value: f32) {
        match param {
            SimParam::MaxVelocity => self.config.physics.max_velocity = value,
            SimParam::CenterPressure => self.config.physics.center_pressure_strength = value,
            SimParam::DeathChance => self.config.reproduction.death_chance_factor = value,
            SimParam::ReproThreshold => {
                self.config.reproduction.reproduction_energy_threshold = value
            }
            SimParam::EnergyCost => self.config.energy.size_energy_cost_factor = value,
            SimParam::BounceFactor => self.config.physics.velocity_bounce_factor = value,
            SimParam::ParticleForce => self.config.physics.particle_force_scale = value,
            SimParam::ParticleFriction => self.config.physics.particle_friction = value,
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

/// Tunable simulation parameters exposed to the UI. Modelling the surface as an
/// enum keeps the JS↔WASM boundary typed: callers cannot pass an unknown
/// parameter, and every variant must be handled in `WebSimulation::update_param`.
#[wasm_bindgen]
pub enum SimParam {
    MaxVelocity,
    CenterPressure,
    DeathChance,
    ReproThreshold,
    EnergyCost,
    BounceFactor,
    ParticleForce,
    ParticleFriction,
}
