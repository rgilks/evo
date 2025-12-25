use serde::{Deserialize, Serialize};

// Core ECS components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Size {
    pub radius: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h * 6.0;
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self {
            r: (r + m).clamp(0.0, 1.0),
            g: (g + m).clamp(0.0, 1.0),
            b: (b + m).clamp(0.0, 1.0),
        }
    }
}

// Movement style components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementStyle {
    pub style: MovementType,
    pub flocking_strength: f32, // How strongly to flock (0.0 = no flocking, 1.0 = strong flocking)
    pub separation_distance: f32, // Preferred distance from other flock members
    pub alignment_strength: f32, // How much to align with flock direction
    pub cohesion_strength: f32, // How much to move toward flock center
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MovementType {
    Random,    // Random movement
    Flocking,  // Flock with similar entities
    Solitary,  // Avoid other entities
    Predatory, // Hunt for prey
    Grazing,   // Move slowly and steadily
}

// Utility structs for better organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Position> for Vec2 {
    fn from(pos: Position) -> Self {
        Vec2::new(pos.x, pos.y)
    }
}

impl From<Velocity> for Vec2 {
    fn from(vel: Velocity) -> Self {
        Vec2::new(vel.x, vel.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position { x: 10.0, y: 20.0 };
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_energy_creation() {
        let energy = Energy {
            current: 50.0,
            max: 100.0,
        };
        assert_eq!(energy.current, 50.0);
        assert_eq!(energy.max, 100.0);
    }

    #[test]
    fn test_size_creation() {
        let size = Size { radius: 15.0 };
        assert_eq!(size.radius, 15.0);
    }

    #[test]
    fn test_velocity_creation() {
        let vel = Velocity { x: 5.0, y: -3.0 };
        assert_eq!(vel.x, 5.0);
        assert_eq!(vel.y, -3.0);
    }

    #[test]
    fn test_color_creation() {
        let color = Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_from_hsv() {
        // Test red color
        let red = Color::from_hsv(0.0, 1.0, 1.0);
        assert!((red.r - 1.0).abs() < 0.001);
        assert!((red.g - 0.0).abs() < 0.001);
        assert!((red.b - 0.0).abs() < 0.001);

        // Test green color (120 degrees = 0.33 in HSV)
        let green = Color::from_hsv(0.33, 1.0, 1.0);
        println!(
            "Green HSV(0.33, 1.0, 1.0) -> RGB({:.3}, {:.3}, {:.3})",
            green.r, green.g, green.b
        );
        // Green should have g > r and g > b, and r and b should be close to 0
        assert!(green.g > green.r);
        assert!(green.g > green.b);
        assert!(green.r < 0.1);
        assert!(green.b < 0.1);

        // Test blue color (240 degrees = 0.67 in HSV)
        let blue = Color::from_hsv(0.67, 1.0, 1.0);
        println!(
            "Blue HSV(0.67, 1.0, 1.0) -> RGB({:.3}, {:.3}, {:.3})",
            blue.r, blue.g, blue.b
        );
        // Blue should have b > r and b > g, and r and g should be close to 0
        assert!(blue.b > blue.r);
        assert!(blue.b > blue.g);
        assert!(blue.r < 0.1);
        assert!(blue.g < 0.1);

        // Test white color (saturation = 0)
        let white = Color::from_hsv(0.0, 0.0, 1.0);
        assert!((white.r - 1.0).abs() < 0.001);
        assert!((white.g - 1.0).abs() < 0.001);
        assert!((white.b - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_creation() {
        let vec = Vec2::new(3.0, 4.0);
        assert_eq!(vec.x, 3.0);
        assert_eq!(vec.y, 4.0);
    }

    #[test]
    fn test_vec2_from_position() {
        let pos = Position { x: 10.0, y: 20.0 };
        let vec: Vec2 = pos.into();
        assert_eq!(vec.x, 10.0);
        assert_eq!(vec.y, 20.0);
    }

    #[test]
    fn test_vec2_from_velocity() {
        let vel = Velocity { x: 5.0, y: -3.0 };
        let vec: Vec2 = vel.into();
        assert_eq!(vec.x, 5.0);
        assert_eq!(vec.y, -3.0);
    }

    #[test]
    fn test_components_serialization() {
        let pos = Position { x: 10.0, y: 20.0 };
        let energy = Energy {
            current: 50.0,
            max: 100.0,
        };
        let size = Size { radius: 15.0 };
        let vel = Velocity { x: 5.0, y: -3.0 };
        let color = Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };

        // Test serialization and deserialization
        let pos_serialized = serde_json::to_string(&pos).unwrap();
        let pos_deserialized: Position = serde_json::from_str(&pos_serialized).unwrap();
        assert_eq!(pos.x, pos_deserialized.x);
        assert_eq!(pos.y, pos_deserialized.y);

        let energy_serialized = serde_json::to_string(&energy).unwrap();
        let energy_deserialized: Energy = serde_json::from_str(&energy_serialized).unwrap();
        assert_eq!(energy.current, energy_deserialized.current);
        assert_eq!(energy.max, energy_deserialized.max);

        let size_serialized = serde_json::to_string(&size).unwrap();
        let size_deserialized: Size = serde_json::from_str(&size_serialized).unwrap();
        assert_eq!(size.radius, size_deserialized.radius);

        let vel_serialized = serde_json::to_string(&vel).unwrap();
        let vel_deserialized: Velocity = serde_json::from_str(&vel_serialized).unwrap();
        assert_eq!(vel.x, vel_deserialized.x);
        assert_eq!(vel.y, vel_deserialized.y);

        let color_serialized = serde_json::to_string(&color).unwrap();
        let color_deserialized: Color = serde_json::from_str(&color_serialized).unwrap();
        assert_eq!(color.r, color_deserialized.r);
        assert_eq!(color.g, color_deserialized.g);
        assert_eq!(color.b, color_deserialized.b);
    }

    #[test]
    fn test_components_clone() {
        let pos = Position { x: 10.0, y: 20.0 };
        let energy = Energy {
            current: 50.0,
            max: 100.0,
        };
        let size = Size { radius: 15.0 };
        let vel = Velocity { x: 5.0, y: -3.0 };
        let color = Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };

        let pos_cloned = pos.clone();
        let energy_cloned = energy.clone();
        let size_cloned = size.clone();
        let vel_cloned = vel.clone();
        let color_cloned = color.clone();

        assert_eq!(pos.x, pos_cloned.x);
        assert_eq!(pos.y, pos_cloned.y);
        assert_eq!(energy.current, energy_cloned.current);
        assert_eq!(energy.max, energy_cloned.max);
        assert_eq!(size.radius, size_cloned.radius);
        assert_eq!(vel.x, vel_cloned.x);
        assert_eq!(vel.y, vel_cloned.y);
        assert_eq!(color.r, color_cloned.r);
        assert_eq!(color.g, color_cloned.g);
        assert_eq!(color.b, color_cloned.b);
    }
}
