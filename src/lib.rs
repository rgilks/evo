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

/// Floats packed per entity in the render buffer:
/// prev_x, prev_y, x, y, radius, r, g, b, health, style_id, speed_norm, sense_norm.
const FLOATS_PER_ENTITY: usize = 12;

/// Floats packed per food patch in the render buffer: x, y, radius, intensity_fraction.
const FLOATS_PER_PATCH: usize = 4;

/// Floats packed per visual effect in the render buffer:
/// x, y, base_radius, life, life_step, kind.
const FLOATS_PER_EFFECT: usize = 6;

#[wasm_bindgen]
pub struct WebSimulation {
    simulation: simulation::Simulation,
    seed: u64,
    entity_buffer: Vec<f32>, // Reusable buffer for entity data
    food_buffer: Vec<f32>,   // Reusable buffer for food-patch render data
    effect_buffer: Vec<f32>, // Reusable buffer for transient visual effects
}

#[wasm_bindgen]
impl WebSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(world_size: f32, config_json: &str) -> Result<WebSimulation, JsValue> {
        // Seed from the wall clock so each page load is a different run.
        let seed = js_sys::Date::now() as u64;
        Self::build(world_size, config_json, seed)
    }

    /// Construct with an explicit seed so a run can be reproduced or shared.
    /// `seed` arrives as an f64 from JS — exact for the wall-clock seeds we log
    /// (well below 2^53).
    pub fn with_seed(
        world_size: f32,
        config_json: &str,
        seed: f64,
    ) -> Result<WebSimulation, JsValue> {
        Self::build(world_size, config_json, seed as u64)
    }

    /// The seed this run was created from (as f64 for JS; exact below 2^53).
    pub fn get_seed(&self) -> f64 {
        self.seed as f64
    }

    pub fn update(&mut self) {
        self.simulation.update();
    }

    /// Update entity buffer and return pointer for WebGPU renderer
    pub fn update_entity_buffer(&mut self) -> *const f32 {
        let entity_tuples = self.simulation.get_entities();
        self.entity_buffer.clear();

        for (px, py, cx, cy, radius, r, g, b, health, style, speed, sense) in entity_tuples {
            self.entity_buffer.push(px);
            self.entity_buffer.push(py);
            self.entity_buffer.push(cx);
            self.entity_buffer.push(cy);
            self.entity_buffer.push(radius);
            self.entity_buffer.push(r);
            self.entity_buffer.push(g);
            self.entity_buffer.push(b);
            self.entity_buffer.push(health);
            self.entity_buffer.push(style);
            self.entity_buffer.push(speed);
            self.entity_buffer.push(sense);
        }

        self.entity_buffer.as_ptr()
    }

    /// Number of entities in the last buffered frame (from `update_entity_buffer`).
    pub fn entity_count(&self) -> u32 {
        (self.entity_buffer.len() / FLOATS_PER_ENTITY) as u32
    }

    /// Update the food-patch buffer and return a pointer for the WebGPU renderer.
    /// Layout per patch: x, y, radius, intensity_fraction (4 floats).
    pub fn update_food_buffer(&mut self) -> *const f32 {
        let patches = self.simulation.get_food_patches();
        self.food_buffer.clear();
        for (x, y, radius, intensity) in patches {
            self.food_buffer.push(x);
            self.food_buffer.push(y);
            self.food_buffer.push(radius);
            self.food_buffer.push(intensity);
        }
        self.food_buffer.as_ptr()
    }

    /// Number of food patches in the last buffered frame (from `update_food_buffer`).
    pub fn food_count(&self) -> u32 {
        (self.food_buffer.len() / FLOATS_PER_PATCH) as u32
    }

    /// Update the effect buffer and return a pointer for the WebGPU renderer.
    /// Layout per effect: x, y, base_radius, life, life_step, kind (6 floats).
    pub fn update_effect_buffer(&mut self) -> *const f32 {
        let effects = self.simulation.get_effects();
        self.effect_buffer.clear();
        for (x, y, radius, life, life_step, kind) in effects {
            self.effect_buffer.push(x);
            self.effect_buffer.push(y);
            self.effect_buffer.push(radius);
            self.effect_buffer.push(life);
            self.effect_buffer.push(life_step);
            self.effect_buffer.push(kind);
        }
        self.effect_buffer.as_ptr()
    }

    /// Number of effects in the last buffered frame (from `update_effect_buffer`).
    pub fn effect_count(&self) -> u32 {
        (self.effect_buffer.len() / FLOATS_PER_EFFECT) as u32
    }

    pub fn get_stats(&self) -> JsValue {
        let config = self.simulation.config();
        let stats = stats::SimulationStats::from_world(
            self.simulation.world(),
            config.population.max_population as f32,
            config.population.entity_scale,
        );
        serde_wasm_bindgen::to_value(&stats).unwrap_or_else(|e| {
            web_sys::console::error_1(&JsValue::from_str(&format!(
                "get_stats serialization failed: {e}"
            )));
            JsValue::NULL
        })
    }

    pub fn get_world_size(&self) -> f32 {
        self.simulation.world_size()
    }

    /// Centroid + focus radius of the live population as `[cx, cy, radius]` in
    /// world units, so the UI can frame the swarm (returned to JS as a
    /// Float32Array).
    pub fn get_view_focus(&self) -> Vec<f32> {
        let (cx, cy, r) = self.simulation.view_focus();
        vec![cx, cy, r]
    }

    pub fn update_param(&mut self, param: SimParam, value: f32) {
        // Mutate the single config the Simulation owns — no duplicate, no clone.
        let config = self.simulation.config_mut();
        match param {
            SimParam::MaxVelocity => config.physics.max_velocity = value,
            SimParam::EdgeRepulsion => config.physics.edge_repulsion_strength = value,
            SimParam::DeathChance => config.reproduction.death_chance_factor = value,
            SimParam::ReproThreshold => config.reproduction.reproduction_energy_threshold = value,
            SimParam::EnergyCost => config.energy.size_energy_cost_factor = value,
            SimParam::BounceFactor => config.physics.velocity_bounce_factor = value,
            SimParam::ParticleForce => config.physics.particle_force_scale = value,
            SimParam::ParticleFriction => config.physics.particle_friction = value,
            SimParam::Food => config.energy.ambient_energy_gain = value,
            SimParam::Predation => config.physics.interaction_radius_offset = value,
        }
    }

    pub fn get_step(&self) -> u32 {
        self.simulation.step()
    }

    /// Instantly cull a fraction of the population (user action, immediate effect).
    pub fn cull(&mut self, fraction: f32) {
        self.simulation.cull(fraction);
    }

    /// Instantly spawn a burst of new creatures (user action, immediate effect).
    pub fn bloom(&mut self, count: u32) {
        self.simulation.bloom(count);
    }

    /// Spawn a burst of new creatures at world `(x, y)` — the user clicking the
    /// canvas to seed life at the cursor.
    pub fn bloom_at(&mut self, x: f32, y: f32, count: u32) {
        self.simulation.bloom_at(x, y, count);
    }

    /// Drop a patch of food at world `(x, y)` — the user clicking to feed the
    /// world; creatures swarm it and graze it down until it disappears.
    pub fn drop_food(&mut self, x: f32, y: f32) {
        self.simulation.drop_food(x, y);
    }
}

impl WebSimulation {
    fn build(world_size: f32, config_json: &str, seed: u64) -> Result<WebSimulation, JsValue> {
        let config: config::SimulationConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;

        let simulation = simulation::Simulation::new_with_config_seeded(world_size, config, seed);

        Ok(WebSimulation {
            simulation,
            seed,
            entity_buffer: Vec::with_capacity(120000), // 10000 entities * 12 floats
            food_buffer: Vec::with_capacity(64),       // a handful of patches * 4 floats
            effect_buffer: Vec::with_capacity(2400),   // up to ~400 effects * 6 floats
        })
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
    EdgeRepulsion,
    DeathChance,
    ReproThreshold,
    EnergyCost,
    BounceFactor,
    ParticleForce,
    ParticleFriction,
    Food,
    Predation,
}
