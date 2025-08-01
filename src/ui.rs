use crate::simulation::Simulation;
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

pub fn run(world_size: f32) {
    let mut window = Window::new(
        "Evolution Simulation - Press ESC to exit",
        800,
        600,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Set to 60 fps for smooth animation and maximum CPU utilization
    window.set_target_fps(60);

    let mut simulation = Simulation::new(world_size);
    let mut frame_count = 0;
    let mut last_redraw = Instant::now();

    println!("Evolution simulation window created! You should see colored circles representing entities.");

    // Create a buffer for the window
    let mut buffer: Vec<u32> = vec![0; 800 * 600];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update simulation every frame to maximize CPU utilization
        simulation.update();
        frame_count += 1;

        // Redraw every frame at 60 FPS
        let now = Instant::now();
        if now.duration_since(last_redraw).as_millis() >= 16 {
            last_redraw = now;

            let entities = simulation.get_entities();

            // Debug: Print frame info
            if frame_count % 60 == 0 {
                println!("Frame {}: {} entities", frame_count, entities.len());
            }

            // Clear the buffer with dark background
            for pixel in &mut buffer {
                *pixel = 0x1a1a1a; // Dark gray
            }

            // Draw more entities to show the work being done
            let max_entities_to_draw = 1000; // Increased to show more work
            let entities_to_draw = if entities.len() > max_entities_to_draw {
                // Sample entities evenly across the population
                let step = entities.len() / max_entities_to_draw;
                entities
                    .iter()
                    .step_by(step)
                    .take(max_entities_to_draw)
                    .collect::<Vec<_>>()
            } else {
                entities.iter().collect::<Vec<_>>()
            };

            // Draw entities as circles
            for (x, y, radius, r, g, b) in entities_to_draw {
                // Convert world coordinates to screen coordinates
                let screen_x = ((*x + world_size / 2.0) / world_size * 800.0) as i32;
                let screen_y = ((*y + world_size / 2.0) / world_size * 600.0) as i32;
                let screen_radius = (*radius / world_size * 800.0_f32.min(600.0)) as i32;

                // Cull entities that are off-screen
                if screen_x < -screen_radius
                    || screen_x > 800 + screen_radius
                    || screen_y < -screen_radius
                    || screen_y > 600 + screen_radius
                {
                    continue;
                }

                // Convert colors from 0-1 to 0-255
                let color_r = (*r * 255.0) as u8;
                let color_g = (*g * 255.0) as u8;
                let color_b = (*b * 255.0) as u8;

                // Create color in RGB format
                let color = (color_r as u32) << 16 | (color_g as u32) << 8 | color_b as u32;

                // Draw circle using optimized algorithm
                draw_circle_optimized(
                    &mut buffer,
                    800,
                    600,
                    screen_x,
                    screen_y,
                    screen_radius.max(1),
                    color,
                );
            }

            // Update the window with the buffer
            window.update_with_buffer(&buffer, 800, 600).unwrap();
        }
    }

    println!("Simulation window closed");
}

fn draw_circle_optimized(
    buffer: &mut [u32],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: u32,
) {
    let width = width as i32;
    let height = height as i32;
    let radius_squared = radius * radius;

    // Use Bresenham's circle algorithm for better performance
    let mut x = radius;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        // Draw 8 octants of the circle
        let points = [
            (center_x + x, center_y + y),
            (center_x + y, center_y + x),
            (center_x - y, center_y + x),
            (center_x - x, center_y + y),
            (center_x - x, center_y - y),
            (center_x - y, center_y - x),
            (center_x + y, center_y - x),
            (center_x + x, center_y - y),
        ];

        for (px, py) in points {
            if px >= 0 && px < width && py >= 0 && py < height {
                let index = (py * width + px) as usize;
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }

        if err <= 0 {
            y += 1;
            err += 2 * y + 1;
        }
        if err > 0 {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}

fn draw_circle(
    buffer: &mut [u32],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: u32,
) {
    let width = width as i32;
    let height = height as i32;

    // Simple circle drawing algorithm
    for y in (center_y - radius).max(0)..(center_y + radius).min(height) {
        for x in (center_x - radius).max(0)..(center_x + radius).min(width) {
            let dx = x - center_x;
            let dy = y - center_y;
            let distance_squared = dx * dx + dy * dy;

            if distance_squared <= radius * radius {
                let index = (y * width + x) as usize;
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }
    }
}
