import init, {
  initThreadPool,
  WebSimulation,
  WebGpuRenderer,
  init_panic_hook,
  SimParam,
} from "../pkg/evo.js?v=bab2bc7";

// Shared configuration object - matches the new Rust SimulationConfig structure
const DEFAULT_CONFIG = {
  population: {
    entity_scale: 0.5,
    max_population: 10000,
    initial_entities: 2500,
    spawn_radius_factor: 0.2,
  },
  physics: {
    max_velocity: 2.0,
    max_entity_radius: 20.0,
    min_entity_radius: 1.0,
    grid_cell_size: 25.0,
    boundary_margin: 5.0,
    interaction_radius_offset: 6.0,
    velocity_bounce_factor: 0.8,
    edge_repulsion_strength: 0.3,
    particle_force_scale: 0.15,
    particle_friction: 0.95,
  },
  energy: {
    size_energy_cost_factor: 0.15,
    movement_energy_cost: 0.1,
    // Richer field lifts the carrying capacity so the boom/bust waves play out
    // well above the safety floor (see Rust SimulationConfig::default).
    ambient_energy_gain: 1.3,
    predator_graze_fraction: 0.6,
    predator_upkeep: 0.0,
  },
  reproduction: {
    reproduction_energy_threshold: 0.6,
    reproduction_energy_cost: 0.7,
    child_energy_factor: 0.4,
    child_spawn_radius: 15.0,
    population_density_factor: 0.8,
    min_reproduction_chance: 0.05,
    death_chance_factor: 0.04,
    // Lagged-mortality boom/bust + its safety floor, and the speciation throttle.
    crowding_pressure_rate: 0.006,
    death_floor_density: 0.03,
    hue_crowding_factor: 1.2,
  },
};

// Serialized once at module load — the config is constant, so init() and reset()
// reuse it instead of re-stringifying on every (re)construction.
const CONFIG_JSON = JSON.stringify(DEFAULT_CONFIG);

// Ecosystem + motion sliders, each mapping one DOM slider to one SimParam.
// Adding a slider is a one-line entry here plus the markup in index.html.
const SLIDERS = [
  { id: "max-velocity", valueId: "velocity-value", param: SimParam.MaxVelocity, decimals: 1 },
  { id: "edge-repulsion", valueId: "pressure-value", param: SimParam.EdgeRepulsion, decimals: 2 },
  { id: "death-chance", valueId: "death-value", param: SimParam.DeathChance, decimals: 2 },
  { id: "repro-threshold", valueId: "repro-value", param: SimParam.ReproThreshold, decimals: 2 },
  { id: "energy-cost", valueId: "energy-value", param: SimParam.EnergyCost, decimals: 2 },
  { id: "particle-force", valueId: "pforce-value", param: SimParam.ParticleForce, decimals: 1 },
  { id: "particle-friction", valueId: "pfriction-value", param: SimParam.ParticleFriction, decimals: 2 },
  { id: "ambient-energy", valueId: "food-value", param: SimParam.Food, decimals: 2 },
  { id: "predation-reach", valueId: "predation-value", param: SimParam.Predation, decimals: 0 },
];

class EvolutionApp {
  constructor() {
    this.simulation = null;
    this.renderer = null;
    this.canvas = null;
    this.animationId = null;
    this.lastTime = 0;
    this.frameCount = 0;
    this.fps = 0;
    // Render-upload gating: the instance buffer is repacked only when the sim
    // step advances (see render()); other frames re-use the cached pointer.
    this.lastRenderedStep = -1;
    this.entityPtr = 0;
    this.entityCount = 0;
    this.foodPtr = 0;
    this.foodCount = 0;
    // Simulation ticks per second. The renderer runs at full refresh rate and
    // interpolates between ticks, so a low sim rate gives smooth, fluid, slow
    // motion (and far less birth/death flicker) rather than 60 jumps a second.
    this.targetFPS = 15;

    // Camera state
    this.camera = {
      zoom: 1.0,
      x: 0.0,
      y: 0.0,
      isPanning: false,
      lastMouseX: 0,
      lastMouseY: 0,
    };

    this.init();
  }

  async init() {
    try {
      // Check for SharedArrayBuffer support (required for wasm-bindgen-rayon)
      if (!window.SharedArrayBuffer) {
        throw new Error(
          "SharedArrayBuffer is not supported. Ensure Cross-Origin Isolation (COOP/COEP) headers are set correctly."
        );
      }

      // Initialize WASM
      await init();
      init_panic_hook();

      // Initialize thread pool
      await initThreadPool(navigator.hardwareConcurrency);

      // Get the canvas and build the simulation (sizing + seed in one place).
      this.canvas = document.getElementById("simulation-canvas");
      this.createSimulation();

      // Initialize WebGPU renderer (required - no fallback)
      if (!navigator.gpu) {
        throw new Error("WebGPU is required but not available in this browser");
      }
      console.log("Initializing WebGPU renderer...");
      this.renderer = await WebGpuRenderer.create(this.canvas);
      console.log("WebGPU renderer initialized successfully!");

      this.setupEventListeners();
      this.startRenderLoop();
    } catch (error) {
      console.error("Failed to initialize:", error);
      this.showError("Failed to initialize simulation: " + error.message);
    }
  }

  setupEventListeners() {
    // Control panel: a ⚙ handle toggles it (tap) and drags it (press + move),
    // matching the galacto sandbox. The dragged position persists across loads.
    const panel = document.getElementById("controls");
    const handle = document.getElementById("controls-toggle");
    const setCollapsed = (collapsed) => {
      panel.classList.toggle("collapsed", collapsed);
      handle.setAttribute("aria-expanded", String(!collapsed));
      handle.title = collapsed
        ? "Show controls (drag to move)"
        : "Hide controls (drag to move)";
    };
    const moveTo = (left, top) => {
      panel.style.left = left + "px";
      panel.style.top = top + "px";
      panel.style.right = "auto";
    };
    const clampIntoView = () => {
      if (!panel.style.left) return;
      const r = panel.getBoundingClientRect();
      moveTo(
        Math.max(4, Math.min(parseFloat(panel.style.left), window.innerWidth - r.width - 4)),
        Math.max(4, Math.min(parseFloat(panel.style.top), window.innerHeight - r.height - 4))
      );
    };
    try {
      const saved = JSON.parse(localStorage.getItem("evo-panel-pos") || "null");
      if (saved && Number.isFinite(saved.left) && Number.isFinite(saved.top)) {
        moveTo(saved.left, saved.top);
        clampIntoView();
      }
    } catch {}
    let startX, startY, baseLeft, baseTop, dragging = false, moved = false;
    handle.addEventListener("pointerdown", (e) => {
      dragging = true;
      moved = false;
      const r = panel.getBoundingClientRect();
      baseLeft = r.left;
      baseTop = r.top;
      startX = e.clientX;
      startY = e.clientY;
      handle.setPointerCapture(e.pointerId);
    });
    handle.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (!moved && Math.hypot(dx, dy) > 4) moved = true;
      if (!moved) return;
      moveTo(baseLeft + dx, baseTop + dy);
      clampIntoView();
    });
    const endDrag = (e) => {
      if (!dragging) return;
      dragging = false;
      try {
        handle.releasePointerCapture(e.pointerId);
      } catch {}
      if (moved) {
        try {
          localStorage.setItem(
            "evo-panel-pos",
            JSON.stringify({
              left: parseFloat(panel.style.left),
              top: parseFloat(panel.style.top),
            })
          );
        } catch {}
      }
    };
    handle.addEventListener("pointerup", endDrag);
    handle.addEventListener("pointercancel", endDrag);
    handle.addEventListener("click", () => {
      if (moved) {
        moved = false;
        return;
      }
      setCollapsed(!panel.classList.contains("collapsed"));
      clampIntoView();
    });

    document.getElementById("reset").addEventListener("click", () => this.reset());

    // Instant actions — immediate, visible effect on the population without
    // touching the (deliberately gradual) ecosystem balance.
    document.getElementById("cull").addEventListener("click", () => {
      this.simulation.cull(0.5);
      this.lastRenderedStep = -1; // force a repack so the change shows at once
      this.updateStats();
    });
    document.getElementById("bloom").addEventListener("click", () => {
      this.simulation.bloom(500);
      this.lastRenderedStep = -1;
      this.updateStats();
    });

    // Parameter sliders (table-driven — see SLIDERS) plus the sim-speed control.
    this.setupSliders();

    // Keyboard shortcuts (ignored while typing in the seed box)
    document.addEventListener("keydown", (e) => {
      if (e.target.tagName === "INPUT") return;
      if (e.key === "h" || e.key === "H") {
        setCollapsed(!panel.classList.contains("collapsed"));
        clampIntoView();
      } else if (e.key === "Escape") {
        setCollapsed(true);
      } else if (e.key === "r" || e.key === "R") {
        this.reset();
      }
    });

    // Mouse Controls (Zoom and Pan)
    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      const zoomSpeed = 0.001;
      const factor = Math.exp(-e.deltaY * zoomSpeed);
      this.camera.zoom *= factor;
      this.camera.zoom = Math.min(Math.max(this.camera.zoom, 0.1), 10.0);
    });

    this.canvas.addEventListener("mousedown", (e) => {
      if (e.button === 0) {
        // Left click to pan
        this.camera.isPanning = true;
        this.camera.lastMouseX = e.clientX;
        this.camera.lastMouseY = e.clientY;
      }
    });

    window.addEventListener("mousemove", (e) => {
      if (this.camera.isPanning) {
        const dx = (e.clientX - this.camera.lastMouseX) / (this.canvas.width / 2);
        const dy = (e.clientY - this.camera.lastMouseY) / (this.canvas.height / 2);

        this.camera.x += dx / this.camera.zoom;
        this.camera.y -= dy / this.camera.zoom;

        this.camera.lastMouseX = e.clientX;
        this.camera.lastMouseY = e.clientY;
      }
    });

    window.addEventListener("mouseup", (e) => {
      if (e.button === 0) {
        this.camera.isPanning = false;
      }
    });
  }

  // Wire every parameter slider from the SLIDERS table; the sim-speed slider is
  // special — it sets the tick rate, not a SimParam.
  setupSliders() {
    for (const s of SLIDERS) {
      const el = document.getElementById(s.id);
      if (!el) continue;
      el.addEventListener("input", (e) => {
        const value = parseFloat(e.target.value);
        document.getElementById(s.valueId).textContent = value.toFixed(s.decimals);
        this.simulation.update_param(s.param, value);
      });
    }

    // Sim Speed sets the simulation tick rate (ticks/sec); the renderer keeps
    // interpolating at full refresh rate, so lower = slower, smoother motion.
    const simSpeedSlider = document.getElementById("sim-speed");
    simSpeedSlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("sim-speed-value").textContent = value.toFixed(0);
      this.targetFPS = value;
    });
  }

  // Size the canvas to the window and build a fresh simulation. The seed box, if
  // set, reproduces a specific run; empty means a wall-clock seed. Shared by
  // init() and reset() so construction lives in one place.
  createSimulation() {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
    const worldSize = Math.max(this.canvas.width, this.canvas.height);
    const seedText = document.getElementById("seed-input")?.value.trim();
    const seed = seedText ? Number(seedText) : NaN;
    this.simulation = Number.isFinite(seed)
      ? WebSimulation.with_seed(worldSize, CONFIG_JSON, seed)
      : new WebSimulation(worldSize, CONFIG_JSON);
    this.updateSeedDisplay();
    this.lastRenderedStep = -1; // fresh sim — force a repack on the next frame
  }

  reset() {
    this.createSimulation();
    this.updateStats();
  }

  updateSeedDisplay() {
    const el = document.getElementById("seed-display");
    if (el && this.simulation) {
      el.textContent = String(this.simulation.get_seed());
    }
  }

  startRenderLoop() {
    const animate = (currentTime) => {
      // Calculate FPS
      this.frameCount++;
      if (currentTime - this.lastTime >= 1000) {
        this.fps = this.frameCount;
        this.frameCount = 0;
        this.lastTime = currentTime;
        this.updateStats();
      }

      // Update simulation at target FPS
      const targetInterval = 1000 / this.targetFPS;
      if (currentTime - this.lastUpdateTime >= targetInterval) {
        this.simulation.update();
        this.lastUpdateTime = currentTime;
      }

      // Render
      this.render();

      this.animationId = requestAnimationFrame(animate);
    };

    this.lastUpdateTime = performance.now();
    this.animationId = requestAnimationFrame(animate);
  }

  render() {
    if (!this.simulation || !this.renderer) return;

    // Repack/upload the instance buffer only when the sim actually ticked. On
    // in-between frames the positions are unchanged — only the interpolation
    // factor moves — so the cached buffer is re-rendered. cull/bloom/reset set
    // lastRenderedStep to -1 to force a repack on the next frame.
    const step = this.simulation.get_step();
    if (step !== this.lastRenderedStep) {
      this.entityPtr = this.simulation.update_entity_buffer();
      this.entityCount = this.simulation.entity_count();
      // Food patches drift slowly; repack them on the same cadence as entities.
      this.foodPtr = this.simulation.update_food_buffer();
      this.foodCount = this.simulation.food_count();
      this.lastRenderedStep = step;
    }

    const worldSize = this.simulation.get_world_size();
    const targetInterval = 1000 / this.targetFPS;
    const currentTime = performance.now();
    const interpolationFactor = Math.min(1.0, (currentTime - this.lastUpdateTime) / targetInterval);

    this.renderer.render(
      this.entityPtr,
      this.entityCount,
      this.foodPtr,
      this.foodCount,
      worldSize,
      interpolationFactor,
      this.camera.zoom,
      this.camera.x,
      this.camera.y
    );
  }

  updateStats() {
    if (this.simulation) {
      const stats = this.simulation.get_stats();
      if (stats) {
        document.getElementById("population").textContent =
          stats.total_entities || 0;
        document.getElementById("step-count").textContent =
          this.simulation.get_step() || 0;
        document.getElementById("fps").textContent = this.fps;
      }
    }
  }

  showError(message) {
    const errorDiv = document.createElement("div");
    errorDiv.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #f44336;
            color: white;
            padding: 15px;
            border-radius: 5px;
            z-index: 1000;
        `;
    errorDiv.textContent = message;
    document.body.appendChild(errorDiv);

    setTimeout(() => {
      document.body.removeChild(errorDiv);
    }, 5000);
  }
}

// Start the application when the page loads
window.addEventListener("load", () => {
  const app = new EvolutionApp();

  // Handle window resize
  window.addEventListener("resize", () => {
    const canvas = document.getElementById("simulation-canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    if (app.renderer) {
      app.renderer.resize(canvas.width, canvas.height);
    }
  });
});
