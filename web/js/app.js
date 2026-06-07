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
    max_velocity: 6.0,
    max_entity_radius: 20.0,
    min_entity_radius: 1.0,
    grid_cell_size: 25.0,
    boundary_margin: 5.0,
    interaction_radius_offset: 8.0,
    velocity_bounce_factor: 0.8,
    edge_repulsion_strength: 0.3,
    particle_force_scale: 0.15,
    particle_friction: 0.95,
  },
  energy: {
    size_energy_cost_factor: 0.36,
    movement_energy_cost: 0.1,
    // Curated live default — a leaner field than the Rust default's 1.3, tuned
    // by eye for the look on the site (the browser passes its own config).
    ambient_energy_gain: 0.35,
    predator_graze_fraction: 0.6,
    predator_upkeep: 0.0,
  },
  reproduction: {
    reproduction_energy_threshold: 0.45,
    reproduction_energy_cost: 0.7,
    child_energy_factor: 0.4,
    child_spawn_radius: 15.0,
    population_density_factor: 0.8,
    min_reproduction_chance: 0.05,
    death_chance_factor: 0.11,
    // Lagged-mortality boom/bust + its safety floor, and the speciation throttle.
    crowding_pressure_rate: 0.006,
    death_floor_density: 0.03,
    hue_crowding_factor: 1.2,
  },
  food: {
    // Tuned for a visibly self-sustaining field: patches regrow gently when
    // uneaten and are drawn down fast when a crowd grazes them (stronger graze,
    // slower regrow, lower floor than the Rust defaults). Browser-only — the
    // Rust FoodConfig::default the tests rely on is untouched.
    patch_count: 7,
    patch_radius_frac: 0.11,
    drift_speed: 0.0002,
    regen_rate: 0.015,
    graze_rate: 0.03,
    seek_strength: 1.0,
    patch_fraction: 0.35,
    graze_floor: 0.05,
  },
};

// Serialized once at module load — the config is constant, so init() and reset()
// reuse it instead of re-stringifying on every (re)construction.
const CONFIG_JSON = JSON.stringify(DEFAULT_CONFIG);
const DEFAULT_SEED = 12345;

// Ecosystem + motion sliders, each mapping one DOM slider to one SimParam.
// Adding a slider is a one-line entry here plus the markup in index.html.
const SLIDERS = [
  { id: "edge-repulsion", valueId: "pressure-value", param: SimParam.EdgeRepulsion, decimals: 2 },
  { id: "death-chance", valueId: "death-value", param: SimParam.DeathChance, decimals: 2 },
  { id: "repro-threshold", valueId: "repro-value", param: SimParam.ReproThreshold, decimals: 2 },
  { id: "energy-cost", valueId: "energy-value", param: SimParam.EnergyCost, decimals: 2 },
  { id: "particle-force", valueId: "pforce-value", param: SimParam.ParticleForce, decimals: 1 },
  { id: "particle-friction", valueId: "pfriction-value", param: SimParam.ParticleFriction, decimals: 2 },
  { id: "ambient-energy", valueId: "food-value", param: SimParam.Food, decimals: 2 },
  { id: "predation-reach", valueId: "predation-value", param: SimParam.Predation, decimals: 0 },
];

// Visual sliders — purely cosmetic, applied instantly via renderer.set_visual_params
// so dragging one is immediately visible (the delight controls). `key` indexes the
// app's `visual` state object.
const VISUAL_SLIDERS = [
  { id: "glow", valueId: "glow-value", key: "glow", decimals: 2 },
  { id: "trails", valueId: "trails-value", key: "trails", decimals: 3 },
  { id: "brightness", valueId: "brightness-value", key: "brightness", decimals: 2 },
  { id: "creature-size", valueId: "size-value", key: "size", decimals: 2 },
];

// Generative, fully-synthesised soundscape driven by the simulation — no samples.
// A six-voice drone whose chord is the on-screen hue palette (each hue sector =
// one voice), with brightness from the population's health, body from its size,
// through a procedurally-generated reverb and a slow feedback delay. Created
// lazily on first enable, since browsers require a user gesture to start audio.
class AudioEngine {
  constructor() {
    const Ctx = window.AudioContext || window.webkitAudioContext;
    this.ctx = new Ctx();
    const ctx = this.ctx;
    this.volume = 0.45; // the "on" master level (adjustable via ↑/↓)
    this.enabled = false;

    // Master: gain → lowpass (brightness) → gentle compressor → out.
    this.master = ctx.createGain();
    this.master.gain.value = 0.0; // ramped up by setEnabled
    this.brightness = ctx.createBiquadFilter();
    this.brightness.type = "lowpass";
    this.brightness.frequency.value = 1000;
    this.brightness.Q.value = 0.5;
    const comp = ctx.createDynamicsCompressor();
    comp.threshold.value = -20;
    comp.knee.value = 18;
    comp.ratio.value = 3;
    comp.attack.value = 0.01;
    comp.release.value = 0.3;
    this.master.connect(this.brightness);
    this.brightness.connect(comp);
    comp.connect(ctx.destination);

    // Procedural reverb: a generated decaying-noise impulse response (no files).
    this.reverbSend = ctx.createGain();
    const conv = ctx.createConvolver();
    conv.buffer = this._impulse(3.0, 2.4);
    const reverbWet = ctx.createGain();
    reverbWet.gain.value = 0.5;
    this.reverbSend.connect(conv);
    conv.connect(reverbWet);
    reverbWet.connect(this.master);

    // Slow feedback delay, synthesised.
    this.delaySend = ctx.createGain();
    const delay = ctx.createDelay(2.0);
    delay.delayTime.value = 0.5;
    const fb = ctx.createGain();
    fb.gain.value = 0.4;
    const dtone = ctx.createBiquadFilter();
    dtone.type = "lowpass";
    dtone.frequency.value = 1700;
    const delayWet = ctx.createGain();
    delayWet.gain.value = 0.28;
    this.delaySend.connect(delay);
    delay.connect(dtone);
    dtone.connect(fb);
    fb.connect(delay);
    dtone.connect(delayWet);
    delayWet.connect(this.master);

    // Sub drone for body (level follows population).
    this.sub = ctx.createOscillator();
    this.sub.type = "sine";
    this.sub.frequency.value = 55;
    this.subGain = ctx.createGain();
    this.subGain.gain.value = 0.0;
    this.sub.connect(this.subGain);
    this.subGain.connect(this.master);
    this.sub.start();

    // Six hue voices on a calm sus chord (any subset stays consonant). Each is
    // two slightly-detuned oscillators (warmth) → its gain → master + sends.
    const scale = [110.0, 146.83, 164.81, 220.0, 246.94, 329.63]; // A2 D3 E3 A3 B3 E4
    this.voiceGains = [];
    for (let i = 0; i < scale.length; i++) {
      const vg = ctx.createGain();
      vg.gain.value = 0.0;
      const o1 = ctx.createOscillator();
      o1.type = "sine";
      o1.frequency.value = scale[i];
      o1.detune.value = -5;
      const o2 = ctx.createOscillator();
      o2.type = "triangle";
      o2.frequency.value = scale[i];
      o2.detune.value = 6;
      o1.connect(vg);
      o2.connect(vg);
      vg.connect(this.master);
      vg.connect(this.reverbSend);
      vg.connect(this.delaySend);
      o1.start();
      o2.start();
      this.voiceGains.push(vg);
    }

    // Slow breathing LFO on the master brightness, added on top of the
    // health-driven cutoff for a living feel.
    this.lfo = ctx.createOscillator();
    this.lfo.frequency.value = 0.06;
    const lfoGain = ctx.createGain();
    lfoGain.gain.value = 280;
    this.lfo.connect(lfoGain);
    lfoGain.connect(this.brightness.frequency);
    this.lfo.start();
  }

  // A decaying-noise impulse response for the convolver — synthesised, no files.
  _impulse(seconds, decay) {
    const sr = this.ctx.sampleRate;
    const len = Math.floor(sr * seconds);
    const buf = this.ctx.createBuffer(2, len, sr);
    for (let ch = 0; ch < 2; ch++) {
      const d = buf.getChannelData(ch);
      for (let i = 0; i < len; i++) {
        d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, decay);
      }
    }
    return buf;
  }

  async setEnabled(on) {
    this.enabled = on;
    if (on && this.ctx.state !== "running") {
      try {
        await this.ctx.resume();
      } catch {}
    }
    const t = this.ctx.currentTime;
    this.master.gain.cancelScheduledValues(t);
    this.master.gain.setTargetAtTime(on ? this.volume : 0.0, t, 0.4);
  }

  // Nudge the master volume (the "on" level) and apply it if currently audible.
  adjustVolume(delta) {
    this.volume = Math.min(Math.max(this.volume + delta, 0), 1);
    if (this.enabled) {
      this.master.gain.setTargetAtTime(this.volume, this.ctx.currentTime, 0.15);
    }
    return this.volume;
  }

  // features = [population, avgHealth, hueBin0 .. hueBin5] from audio_features().
  update(features) {
    if (!features || features.length < 8) return;
    const t = this.ctx.currentTime;
    const pop = features[0];
    const health = Math.min(Math.max(features[1], 0), 1);
    const popLevel = Math.min(pop / 600, 1);
    // Health opens the filter; population scales how far it can open.
    const cutoff = 320 + 2600 * health * (0.4 + 0.6 * popLevel);
    this.brightness.frequency.setTargetAtTime(cutoff, t, 0.6);
    this.subGain.gain.setTargetAtTime(0.16 * popLevel, t, 0.4);
    for (let i = 0; i < this.voiceGains.length; i++) {
      const share = features[2 + i] || 0;
      this.voiceGains[i].gain.setTargetAtTime(0.1 * popLevel * Math.pow(share, 0.8), t, 0.4);
    }
  }

  // A short bell when food is dropped: pitch rises toward the top of the screen
  // and it pans with the horizontal position, so each drop sounds like its spot.
  // nx, ny are 0..1 across the canvas (top-left origin).
  ping(nx, ny) {
    if (!this.enabled || this.ctx.state !== "running") return;
    const ctx = this.ctx;
    const t = ctx.currentTime;
    const scale = [523.25, 587.33, 659.25, 783.99, 880.0, 1046.5]; // C5 D5 E5 G5 A5 C6
    const idx = Math.min(scale.length - 1, Math.max(0, Math.floor((1 - ny) * scale.length)));
    const osc = ctx.createOscillator();
    osc.type = "triangle";
    osc.frequency.value = scale[idx];
    const g = ctx.createGain();
    g.gain.value = 0.0001;
    const pan = ctx.createStereoPanner();
    pan.pan.value = Math.min(1, Math.max(-1, nx * 2 - 1));
    osc.connect(g);
    g.connect(pan);
    pan.connect(this.master);
    pan.connect(this.reverbSend);
    g.gain.exponentialRampToValueAtTime(0.28, t + 0.01);
    g.gain.exponentialRampToValueAtTime(0.0006, t + 0.9);
    osc.start(t);
    osc.stop(t + 1.0);
  }
}

class EvolutionApp {
  constructor() {
    this.simulation = null;
    this.renderer = null;
    this.canvas = null;
    this.animationId = null;
    // Render-upload gating: the instance buffer is repacked only when the sim
    // step advances (see render()); other frames re-use the cached pointer.
    this.lastRenderedStep = -1;
    this.entityPtr = 0;
    this.entityCount = 0;
    this.foodPtr = 0;
    this.foodCount = 0;
    this.effectPtr = 0;
    this.effectCount = 0;
    // Simulation ticks per second. The renderer runs at full refresh rate and
    // interpolates between ticks, so a low sim rate gives smooth, fluid, slow
    // motion (and far less birth/death flicker) rather than 60 jumps a second.
    this.targetFPS = 30;

    // Cosmetic visual params (the Visuals sliders), pushed to the renderer via
    // applyVisualParams(). Defaults match the renderer's built-in cinematic look.
    this.visual = { glow: 0.36, trails: 0.875, brightness: 0.8, size: 0.2 };
    // Trait lens: 0=lineage hue, 1=speed, 2=health, 3=behaviour.
    this.colorMode = 0;

    // Generative audio: on by default, but browsers require a user gesture to
    // start a Web Audio context, so the engine is built + resumed on the first
    // interaction (see armAudioAutostart). Toggle with the Sound button or M.
    this.audio = null;
    this.audioEnabled = true;
    this.lastAudioTime = 0;

    // Camera state. The default zoom fills the frame with the settled swarm so
    // the view isn't mostly empty void. `target` is where the camera eases to:
    // programmatic moves (frame/reset) animate toward it, while manual pan/zoom
    // snap both `camera` and `target` together so they never fight the tween.
    this.camera = {
      zoom: 1.5,
      x: 0.0,
      y: 0.0,
      isPanning: false,
      lastMouseX: 0,
      lastMouseY: 0,
    };
    this.cameraTarget = { zoom: 1.5, x: 0.0, y: 0.0 };

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

      // Get the canvas and build the simulation.
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
      this.applyVisualParams(); // push the cinematic defaults into the renderer
      this.armAudioAutostart(); // sound is on by default; starts on first gesture
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
      // Dragging switches off the centred bottom default (CSS left/bottom/transform).
      panel.style.bottom = "auto";
      panel.style.transform = "none";
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
      const saved = JSON.parse(localStorage.getItem("evo-panel-pos-v2") || "null");
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
            "evo-panel-pos-v2",
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
    });
    document.getElementById("bloom").addEventListener("click", () => {
      this.simulation.bloom(500);
      this.lastRenderedStep = -1;
    });

    // Sound toggle. The click is the user gesture that lets the Web Audio context
    // start; the engine is built lazily on first enable.
    const soundBtn = document.getElementById("sound");
    if (soundBtn) {
      soundBtn.addEventListener("click", () => this.toggleSound());
    }

    // Parameter sliders (table-driven — see SLIDERS) plus the sim-speed control.
    this.setupSliders();

    // Keyboard shortcuts (ignored while editing controls)
    document.addEventListener("keydown", (e) => {
      if (e.target.tagName === "INPUT") return;
      if (e.key === "h" || e.key === "H") {
        setCollapsed(!panel.classList.contains("collapsed"));
        clampIntoView();
      } else if (e.key === "Escape") {
        setCollapsed(true);
      } else if (e.key === "r" || e.key === "R") {
        this.reset();
      } else if (e.key === "f" || e.key === "F") {
        this.frame();
      } else if (e.key === "m" || e.key === "M") {
        this.toggleSound();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (this.audio) this.audio.adjustVolume(0.05);
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (this.audio) this.audio.adjustVolume(-0.05);
      }
    });

    // Mouse Controls (Zoom and Pan)
    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      const zoomSpeed = 0.001;
      const factor = Math.exp(-e.deltaY * zoomSpeed);
      this.camera.zoom = Math.min(Math.max(this.camera.zoom * factor, 0.1), 10.0);
      this.cameraTarget.zoom = this.camera.zoom; // manual zoom: no tween lag
    });

    // Left button: a click (no drag) drops food at the cursor; a drag pans.
    // The down position + a small movement threshold tell them apart.
    this.canvas.addEventListener("mousedown", (e) => {
      if (e.button === 0) {
        this.camera.isPanning = true;
        this.camera.lastMouseX = e.clientX;
        this.camera.lastMouseY = e.clientY;
        this.camera.downX = e.clientX;
        this.camera.downY = e.clientY;
        this.camera.moved = false;
      }
    });

    window.addEventListener("mousemove", (e) => {
      if (this.camera.isPanning) {
        if (
          !this.camera.moved &&
          Math.hypot(e.clientX - this.camera.downX, e.clientY - this.camera.downY) > 4
        ) {
          this.camera.moved = true;
        }
        const dx = (e.clientX - this.camera.lastMouseX) / (this.canvas.width / 2);
        const dy = (e.clientY - this.camera.lastMouseY) / (this.canvas.height / 2);

        this.camera.x += dx / this.camera.zoom;
        this.camera.y -= dy / this.camera.zoom;
        // Manual pan is 1:1, so keep the tween target in lockstep.
        this.cameraTarget.x = this.camera.x;
        this.cameraTarget.y = this.camera.y;

        this.camera.lastMouseX = e.clientX;
        this.camera.lastMouseY = e.clientY;
      }
    });

    window.addEventListener("mouseup", (e) => {
      if (e.button === 0 && this.camera.isPanning) {
        this.camera.isPanning = false;
        if (!this.camera.moved) this.dropFoodAtScreen(e.clientX, e.clientY);
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

    // Visual sliders: update local state + the value label, then push to the
    // renderer immediately so the effect is visible as you drag.
    for (const s of VISUAL_SLIDERS) {
      const el = document.getElementById(s.id);
      if (!el) continue;
      el.addEventListener("input", (e) => {
        const value = parseFloat(e.target.value);
        document.getElementById(s.valueId).textContent = value.toFixed(s.decimals);
        this.visual[s.key] = value;
        this.applyVisualParams();
      });
    }

    // Trait lens: cycle what creature colour represents.
    const COLOR_MODES = ["Lineage", "Speed", "Health", "Behaviour"];
    const colorBtn = document.getElementById("color-mode");
    if (colorBtn) {
      colorBtn.addEventListener("click", () => {
        this.colorMode = (this.colorMode + 1) % COLOR_MODES.length;
        colorBtn.textContent = "Colour: " + COLOR_MODES[this.colorMode];
        if (this.renderer) this.renderer.set_color_mode(this.colorMode);
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

  // Size the canvas to the window and build a fresh deterministic simulation.
  // Shared by init() and reset() so construction lives in one place.
  createSimulation() {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
    const worldSize = Math.max(this.canvas.width, this.canvas.height);
    this.simulation = WebSimulation.with_seed(worldSize, CONFIG_JSON, DEFAULT_SEED);
    this.lastRenderedStep = -1; // fresh sim — force a repack on the next frame
  }

  reset() {
    this.createSimulation();
    // Ease back to the default framing as the new world spawns and grows.
    this.cameraTarget = { zoom: 1.5, x: 0.0, y: 0.0 };
  }

  // Smoothly ease the live camera toward its target each frame. Manual pan/zoom
  // keep target == camera, so this only animates programmatic moves (frame/reset).
  updateCamera() {
    const c = this.camera;
    const t = this.cameraTarget;
    const k = 0.1;
    c.zoom += (t.zoom - c.zoom) * k;
    c.x += (t.x - c.x) * k;
    c.y += (t.y - c.y) * k;
  }

  // Frame the live swarm: centre on its centroid and zoom so it fills most of
  // the view. Animated via the camera tween. Bound to the 'f' key.
  frame() {
    if (!this.simulation) return;
    const f = this.simulation.get_view_focus(); // [cx, cy, radius] in world units
    const half = this.simulation.get_world_size() / 2;
    const r = Math.max(f[2], 1e-3);
    // A world point at distance r maps to ndc r/half; fit it to ~78% of the view.
    this.cameraTarget.zoom = Math.min(Math.max((0.78 * half) / r, 0.3), 8.0);
    this.cameraTarget.x = -f[0] / half;
    this.cameraTarget.y = f[1] / half;
  }

  // Drop a patch of food at a screen position (a canvas click). Inverts the
  // shader's world→screen transform (NDC ÷ zoom − camera) × half-world.
  dropFoodAtScreen(clientX, clientY) {
    if (!this.simulation) return;
    const rect = this.canvas.getBoundingClientRect();
    const ndcX = ((clientX - rect.left) / rect.width) * 2 - 1;
    const ndcY = 1 - ((clientY - rect.top) / rect.height) * 2;
    const half = this.simulation.get_world_size() / 2;
    const z = this.camera.zoom;
    const worldX = (ndcX / z - this.camera.x) * half;
    const worldY = (this.camera.y - ndcY / z) * half;
    this.simulation.drop_food(worldX, worldY);
    if (this.audio && this.audioEnabled) {
      // A note that sounds like where you clicked (pitch by height, pan by side).
      this.audio.ping((clientX - rect.left) / rect.width, (clientY - rect.top) / rect.height);
    }
    this.lastRenderedStep = -1; // force a repack so the drop shows at once
  }

  // Toggle the generative soundscape (also bound to the M key). The first enable
  // lazily builds the engine; the triggering click/keypress is the gesture that
  // lets the audio context start.
  async toggleSound() {
    if (!this.audio) {
      try {
        this.audio = new AudioEngine();
      } catch (e) {
        console.error("Audio init failed:", e);
        return;
      }
    }
    this.audioEnabled = !this.audioEnabled;
    await this.audio.setEnabled(this.audioEnabled);
    this.syncSoundButton();
  }

  // Reflect the audio on/off state on the Sound button (highlighted = on).
  syncSoundButton() {
    const btn = document.getElementById("sound");
    if (btn) btn.classList.toggle("active", this.audioEnabled);
  }

  // Audio is on by default, but a Web Audio context can't start until a user
  // gesture — so build + resume the engine on the first interaction.
  armAudioAutostart() {
    const start = () => {
      window.removeEventListener("pointerdown", start);
      window.removeEventListener("keydown", start);
      if (!this.audioEnabled) return;
      if (!this.audio) {
        try {
          this.audio = new AudioEngine();
        } catch (e) {
          console.error("Audio init failed:", e);
          return;
        }
      }
      this.audio.setEnabled(true);
      this.syncSoundButton();
    };
    window.addEventListener("pointerdown", start);
    window.addEventListener("keydown", start);
  }

  // Push the cosmetic visual params (Glow / Trails / Brightness / Size) to the
  // renderer. Cheap, so it's fine to call on every slider input.
  applyVisualParams() {
    if (!this.renderer) return;
    const v = this.visual;
    this.renderer.set_visual_params(v.glow, v.trails, v.brightness, v.size);
  }

  startRenderLoop() {
    const animate = (currentTime) => {
      // Update simulation at target FPS
      const targetInterval = 1000 / this.targetFPS;
      if (currentTime - this.lastUpdateTime >= targetInterval) {
        this.simulation.update();
        this.lastUpdateTime = currentTime;
      }

      // Ease the camera toward its target (no-op when they already match), then
      // render.
      this.updateCamera();
      this.render();

      // Drive the soundscape from the live sim features (~7x/sec; only when on).
      if (this.audioEnabled && this.audio && currentTime - this.lastAudioTime > 150) {
        this.lastAudioTime = currentTime;
        this.audio.update(this.simulation.audio_features());
      }

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
      // Transient effects age every tick, so repack on the same cadence.
      this.effectPtr = this.simulation.update_effect_buffer();
      this.effectCount = this.simulation.effect_count();
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
      this.effectPtr,
      this.effectCount,
      worldSize,
      interpolationFactor,
      this.camera.zoom,
      this.camera.x,
      this.camera.y
    );
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
