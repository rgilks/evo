import init, {
  initThreadPool,
  WebSimulation,
  WebGpuRenderer,
  init_panic_hook,
} from "../pkg/evo.js?v=b250293";

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
    interaction_radius_offset: 15.0,
    velocity_bounce_factor: 0.8,
    center_pressure_strength: 0.3,
  },
  energy: {
    size_energy_cost_factor: 0.15,
    movement_energy_cost: 0.1,
  },
  reproduction: {
    reproduction_energy_threshold: 0.8,
    reproduction_energy_cost: 0.7,
    child_energy_factor: 0.4,
    child_spawn_radius: 15.0,
    population_density_factor: 0.8,
    min_reproduction_chance: 0.05,
    death_chance_factor: 0.1,
  },
};

class EvolutionApp {
  constructor() {
    this.simulation = null;
    this.renderer = null;
    this.canvas = null;
    this.animationId = null;
    this.lastTime = 0;
    this.frameCount = 0;
    this.fps = 0;
    this.targetFPS = 60;

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

      // Get canvas and make it full-screen
      this.canvas = document.getElementById("simulation-canvas");
      this.canvas.width = window.innerWidth;
      this.canvas.height = window.innerHeight;

      const configJson = JSON.stringify(DEFAULT_CONFIG);
      console.log("Config being passed to WebSimulation:", configJson);
      this.simulation = new WebSimulation(
        Math.max(this.canvas.width, this.canvas.height),
        configJson
      );

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
    const resetBtn = document.getElementById("reset");
    const toggleUiBtn = document.getElementById("toggle-ui");
    const showUiBtn = document.getElementById("show-ui-btn");

    resetBtn.addEventListener("click", () => this.reset());
    toggleUiBtn.addEventListener("click", () => this.toggleUI());
    showUiBtn.addEventListener("click", () => this.toggleUI());

    // Parameter sliders
    const velocitySlider = document.getElementById("max-velocity");
    const pressureSlider = document.getElementById("center-pressure");
    const deathSlider = document.getElementById("death-chance");

    velocitySlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("velocity-value").textContent = value.toFixed(1);
      this.simulation.update_param("max_velocity", value);
    });

    pressureSlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("pressure-value").textContent = value.toFixed(2);
      this.simulation.update_param("center_pressure", value);
    });

    deathSlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("death-value").textContent = value.toFixed(2);
      this.simulation.update_param("death_chance", value);
    });

    // New sliders
    const reproSlider = document.getElementById("repro-threshold");
    const energySlider = document.getElementById("energy-cost");
    const bounceSlider = document.getElementById("bounce-factor");

    reproSlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("repro-value").textContent = value.toFixed(2);
      this.simulation.update_param("repro_threshold", value);
    });

    energySlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("energy-value").textContent = value.toFixed(2);
      this.simulation.update_param("energy_cost", value);
    });

    bounceSlider.addEventListener("input", (e) => {
      const value = parseFloat(e.target.value);
      document.getElementById("bounce-value").textContent = value.toFixed(2);
      this.simulation.update_param("bounce_factor", value);
    });

    // Keyboard shortcuts
    document.addEventListener("keydown", (e) => {
      if (e.key === "h" || e.key === "H") {
        this.toggleUI();
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
    // Get canvas and make it full-screen
    const canvas = document.getElementById("simulation-canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    this.simulation = new WebSimulation(
      Math.max(canvas.width, canvas.height),
      JSON.stringify(DEFAULT_CONFIG)
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
      const entityPtr = this.simulation.update_entity_buffer();
      const entityCount = this.simulation.entity_count();
      const worldSize = this.simulation.get_world_size();
      
      // Calculate interpolation factor
      const targetInterval = 1000 / this.targetFPS;
      const currentTime = performance.now();
      const interpolationFactor = Math.min(1.0, (currentTime - this.lastUpdateTime) / targetInterval);
      
      this.renderer.render(
        entityPtr,
        entityCount,
        worldSize,
        interpolationFactor,
        this.camera.zoom,
        this.camera.x,
        this.camera.y
      );
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
    if (app.renderer) {
      app.renderer.resize(canvas.width, canvas.height);
    }
  });
});
