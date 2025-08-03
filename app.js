import init, {
  initThreadPool,
  WebSimulation,
  WebRenderer,
  init_panic_hook,
} from "./pkg/evo.js";

class EvolutionApp {
  constructor() {
    this.simulation = null;
    this.renderer = null;
    this.animationId = null;
    this.lastTime = 0;
    this.frameCount = 0;
    this.fps = 0;
    this.targetFPS = 30;

    this.init();
  }

  async init() {
    try {
      // Initialize WASM
      await init();
      init_panic_hook();

      // Initialize thread pool
      await initThreadPool(navigator.hardwareConcurrency);

      // Create simulation
      const config = {
        entity_scale: 0.5,
        max_population: 5000,
        initial_entities: 3000,
        max_velocity: 2.0,
        max_entity_radius: 20.0,
        min_entity_radius: 1.0,
        spawn_radius_factor: 0.2,
        grid_cell_size: 25.0,
        boundary_margin: 5.0,
        interaction_radius_offset: 15.0,
        reproduction_energy_threshold: 0.8,
        reproduction_energy_cost: 0.7,
        child_energy_factor: 0.4,
        child_spawn_radius: 15.0,
        size_energy_cost_factor: 0.15,
        movement_energy_cost: 0.1,
        population_density_factor: 0.8,
        min_reproduction_chance: 0.05,
        death_chance_factor: 0.1,
        velocity_bounce_factor: 0.8,
      };

      // Get canvas and make it full-screen
      const canvas = document.getElementById("simulation-canvas");
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;

      this.simulation = new WebSimulation(
        Math.max(canvas.width, canvas.height),
        JSON.stringify(config)
      );
      this.renderer = new WebRenderer("simulation-canvas");

      this.setupEventListeners();
      this.startRenderLoop();
    } catch (error) {
      console.error("Failed to initialize:", error);
      this.showError("Failed to initialize simulation");
    }
  }

  setupEventListeners() {
    const resetBtn = document.getElementById("reset");
    const toggleUiBtn = document.getElementById("toggle-ui");
    const showUiBtn = document.getElementById("show-ui-btn");

    resetBtn.addEventListener("click", () => this.reset());
    toggleUiBtn.addEventListener("click", () => this.toggleUI());
    showUiBtn.addEventListener("click", () => this.toggleUI());
    
    // Keyboard shortcuts
    document.addEventListener("keydown", (e) => {
      if (e.key === "h" || e.key === "H") {
        this.toggleUI();
      } else if (e.key === "r" || e.key === "R") {
        this.reset();
      }
    });
  }

  toggleUI() {
    const container = document.querySelector(".container");
    const toggleBtn = document.getElementById("toggle-ui");

    if (container.classList.contains("ui-hidden")) {
      container.classList.remove("ui-hidden");
      toggleBtn.textContent = "Hide UI";
    } else {
      container.classList.add("ui-hidden");
      toggleBtn.textContent = "Show UI";
    }
  }

  reset() {
    // Recreate simulation with same config
    const config = {
      entity_scale: 0.5,
      max_population: 5000,
      initial_entities: 3000,
      max_velocity: 2.0,
      max_entity_radius: 20.0,
      min_entity_radius: 1.0,
      spawn_radius_factor: 0.2,
      grid_cell_size: 25.0,
      boundary_margin: 5.0,
      interaction_radius_offset: 15.0,
      reproduction_energy_threshold: 0.8,
      reproduction_energy_cost: 0.7,
      child_energy_factor: 0.4,
      child_spawn_radius: 15.0,
      size_energy_cost_factor: 0.15,
      movement_energy_cost: 0.1,
      population_density_factor: 0.8,
      min_reproduction_chance: 0.05,
      death_chance_factor: 0.1,
      velocity_bounce_factor: 0.8,
    };

    // Get canvas and make it full-screen
    const canvas = document.getElementById("simulation-canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    this.simulation = new WebSimulation(
      Math.max(canvas.width, canvas.height),
      JSON.stringify(config)
    );
    this.updateStats();
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
    if (this.simulation && this.renderer) {
      const entities = this.simulation.get_entities();
      this.renderer.render(entities);
    }
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
    // Recreate renderer with new canvas size
    app.renderer = new WebRenderer("simulation-canvas");
  });
});
