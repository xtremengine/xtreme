//! # Isometric Coordinate System
//!
//! Utilities for working with isometric projection.
//!
//! ## Isometric Angles
//!
//! Classic isometric uses a 2:1 pixel ratio with camera rotated
//! 45° horizontally and ~35.264° vertically (arctan(1/√2)).

use glam::{Mat4, Vec2, Vec3};

/// Isometric coordinate in tile space
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IsometricCoord {
    /// Column (east-west)
    pub col: i32,
    /// Row (north-south)
    pub row: i32,
    /// Height/layer
    pub height: i32,
}

impl IsometricCoord {
    pub fn new(col: i32, row: i32, height: i32) -> Self {
        Self { col, row, height }
    }

    pub fn flat(col: i32, row: i32) -> Self {
        Self {
            col,
            row,
            height: 0,
        }
    }
}

/// Configuration for isometric projection
#[derive(Clone, Copy, Debug)]
pub struct IsometricConfig {
    /// Tile width in pixels (for 2D) or world units
    pub tile_width: f32,
    /// Tile height in pixels (for 2D) or world units
    pub tile_height: f32,
    /// Height of one vertical unit
    pub height_step: f32,
    /// Camera rotation angle (radians)
    pub camera_yaw: f32,
    /// Camera pitch angle (radians)
    pub camera_pitch: f32,
}

impl Default for IsometricConfig {
    fn default() -> Self {
        Self {
            tile_width: 64.0,
            tile_height: 32.0, // 2:1 ratio
            height_step: 16.0,
            camera_yaw: std::f32::consts::FRAC_PI_4, // 45°
            camera_pitch: (1.0_f32 / 2.0_f32.sqrt()).atan(), // ~35.264°
        }
    }
}

impl IsometricConfig {
    /// Create standard 2:1 isometric config
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create dimetric config (often used in games)
    pub fn dimetric() -> Self {
        Self {
            camera_pitch: 30.0_f32.to_radians(),
            ..Self::default()
        }
    }
}

/// Convert world position to screen position (2D isometric)
///
/// Uses the formula:
/// - screen_x = (world_x - world_y) * tile_width / 2
/// - screen_y = (world_x + world_y) * tile_height / 2 - world_z * height_step
pub fn world_to_screen(world: Vec3, config: &IsometricConfig) -> Vec2 {
    let x = (world.x - world.z) * (config.tile_width / 2.0);
    let y = (world.x + world.z) * (config.tile_height / 2.0) - world.y * config.height_step;
    Vec2::new(x, y)
}

/// Convert screen position to world position (on ground plane y=0)
///
/// Inverse of world_to_screen for y=0 plane
pub fn screen_to_world(screen: Vec2, config: &IsometricConfig) -> Vec3 {
    let half_width = config.tile_width / 2.0;
    let half_height = config.tile_height / 2.0;

    let world_x = (screen.x / half_width + screen.y / half_height) / 2.0;
    let world_z = (screen.y / half_height - screen.x / half_width) / 2.0;

    Vec3::new(world_x, 0.0, world_z)
}

/// Convert tile coordinate to world position
pub fn tile_to_world(coord: IsometricCoord, config: &IsometricConfig) -> Vec3 {
    Vec3::new(
        coord.col as f32,
        coord.height as f32 * config.height_step,
        coord.row as f32,
    )
}

/// Convert world position to tile coordinate (rounds down)
pub fn world_to_tile(world: Vec3, config: &IsometricConfig) -> IsometricCoord {
    IsometricCoord {
        col: world.x.floor() as i32,
        row: world.z.floor() as i32,
        height: (world.y / config.height_step).floor() as i32,
    }
}

/// Create isometric view matrix for 3D camera
///
/// Returns a view matrix that gives isometric-like projection
/// when combined with orthographic projection.
pub fn isometric_view_matrix(config: &IsometricConfig, target: Vec3, distance: f32) -> Mat4 {
    // Calculate camera position based on angles and distance
    let cos_pitch = config.camera_pitch.cos();
    let sin_pitch = config.camera_pitch.sin();
    let cos_yaw = config.camera_yaw.cos();
    let sin_yaw = config.camera_yaw.sin();

    let offset = Vec3::new(
        distance * cos_pitch * sin_yaw,
        distance * sin_pitch,
        distance * cos_pitch * cos_yaw,
    );

    let eye = target + offset;
    Mat4::look_at_rh(eye, target, Vec3::Y)
}

/// Create orthographic projection matrix for isometric view
pub fn isometric_projection_matrix(width: f32, height: f32, near: f32, far: f32) -> Mat4 {
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    Mat4::orthographic_rh(
        -half_width,
        half_width,
        -half_height,
        half_height,
        near,
        far,
    )
}

/// Calculate depth/z-order for sorting sprites in 2D isometric
///
/// Objects further from camera (higher row + col) should be drawn first.
pub fn calculate_depth(coord: IsometricCoord) -> i32 {
    // Simple depth: sum of row and col, adjusted by height
    (coord.row + coord.col) * 100 + coord.height
}

/// Mouse picking helper - get tile under cursor
pub fn pick_tile(
    screen_pos: Vec2,
    screen_size: Vec2,
    config: &IsometricConfig,
    camera_offset: Vec2,
) -> IsometricCoord {
    // Convert screen position to centered coordinates
    let centered = Vec2::new(
        screen_pos.x - screen_size.x / 2.0 + camera_offset.x,
        screen_pos.y - screen_size.y / 2.0 + camera_offset.y,
    );

    let world = screen_to_world(centered, config);
    world_to_tile(world, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_screen_roundtrip() {
        let config = IsometricConfig::default();
        let world = Vec3::new(5.0, 0.0, 3.0);

        let screen = world_to_screen(world, &config);
        let back = screen_to_world(screen, &config);

        assert!((world.x - back.x).abs() < 0.001);
        assert!((world.z - back.z).abs() < 0.001);
    }

    #[test]
    fn test_tile_conversion() {
        let config = IsometricConfig::default();
        let coord = IsometricCoord::new(3, 4, 1);

        let world = tile_to_world(coord, &config);
        let back = world_to_tile(world, &config);

        assert_eq!(coord.col, back.col);
        assert_eq!(coord.row, back.row);
        assert_eq!(coord.height, back.height);
    }

    #[test]
    fn test_depth_ordering() {
        let front = IsometricCoord::new(0, 0, 0);
        let back = IsometricCoord::new(1, 1, 0);
        let high = IsometricCoord::new(0, 0, 1);

        assert!(calculate_depth(front) < calculate_depth(back));
        assert!(calculate_depth(front) < calculate_depth(high));
    }
}
