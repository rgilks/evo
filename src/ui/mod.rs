mod state;

use crate::config::SimulationConfig;
use crate::simulation::Simulation;
use state::State;

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(world_size: f32, config: SimulationConfig) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Evolution Simulation - Press ESC to exit")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);
    let window_for_event = window.clone();

    // Request the first redraw to start the animation loop
    window_for_event.request_redraw();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = instance.create_surface(&window).unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let mut state = pollster::block_on(State::new(&surface, &adapter, window.inner_size()));
    let mut simulation = Simulation::new_with_config(world_size, config);
    let mut frame_count = 0;
    let mut last_frame_time = std::time::Instant::now();
    let mut fps_counter = 0;
    let mut fps_start_time = std::time::Instant::now();
    let _window_id = window.id();

    println!("Evolution simulation window created! You should see colored triangles representing entities.");

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id: event_window_id,
                } if event_window_id == window_for_event.id() => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(&surface, *physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // Frame rate limiting to prevent flickering
                            let now = std::time::Instant::now();
                            let frame_duration = now.duration_since(last_frame_time);
                            if frame_duration.as_millis() < 16 {
                                // ~60 FPS limit
                                return;
                            }
                            last_frame_time = now;

                            // Update simulation every frame for smoother movement
                            simulation.update();
                            frame_count += 1;

                            // Calculate interpolation factor for smooth movement
                            let interpolation_factor = 0.5; // Interpolate halfway between updates

                            // Update rendering every frame for smooth animation
                            state.update_interpolated(
                                &simulation,
                                world_size,
                                interpolation_factor,
                            );

                            // FPS calculation and display
                            fps_counter += 1;
                            if fps_counter >= 60 {
                                let fps_duration = now.duration_since(fps_start_time);
                                let fps = fps_counter as f64 / fps_duration.as_secs_f64();
                                println!(
                                    "FPS: {:.1}, Frame {}: {} entities",
                                    fps,
                                    frame_count,
                                    simulation.get_entities().len()
                                );
                                fps_counter = 0;
                                fps_start_time = now;
                            }

                            // Debug: Print first few entity positions
                            if frame_count % 240 == 0 {
                                let entities = simulation.get_entities();
                                if !entities.is_empty() {
                                    let (x, y, _, _, _, _) = entities[0];
                                    println!("First entity position: ({:.2}, {:.2})", x, y);
                                }
                            }

                            match state.render(&surface) {
                                Ok(_) => {
                                    // Request next frame for continuous animation
                                    // This will be handled by the AboutToWait event
                                }
                                Err(wgpu::SurfaceError::Lost) => state.resize(&surface, state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window_for_event.request_redraw();
                    elwt.set_control_flow(ControlFlow::Poll);
                }
                _ => {}
            }
        })
        .unwrap();
}
