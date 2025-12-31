//! # Spatial Partitioning

use glam::Vec3;
use std::collections::HashMap;
use crate::core::Entity;
use super::shapes::AABB;

/// Grid cell identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCell {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Uniform spatial grid for broadphase collision
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<GridCell, Vec<Entity>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Clear all entities from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Convert world position to grid cell
    pub fn position_to_cell(&self, pos: Vec3) -> GridCell {
        GridCell {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
            z: (pos.z / self.cell_size).floor() as i32,
        }
    }

    /// Insert an entity at a position
    pub fn insert(&mut self, entity: Entity, pos: Vec3) {
        let cell = self.position_to_cell(pos);
        self.cells.entry(cell).or_default().push(entity);
    }

    /// Insert an entity with an AABB (may span multiple cells)
    pub fn insert_aabb(&mut self, entity: Entity, aabb: &AABB) {
        let min_cell = self.position_to_cell(aabb.min);
        let max_cell = self.position_to_cell(aabb.max);

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = GridCell::new(x, y, z);
                    self.cells.entry(cell).or_default().push(entity);
                }
            }
        }
    }

    /// Query entities in a cell
    pub fn query_cell(&self, cell: GridCell) -> &[Entity] {
        self.cells.get(&cell).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Query entities near a position (3x3x3 cells)
    pub fn query_nearby(&self, pos: Vec3) -> Vec<Entity> {
        let center = self.position_to_cell(pos);
        let mut result = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell = GridCell::new(center.x + dx, center.y + dy, center.z + dz);
                    if let Some(entities) = self.cells.get(&cell) {
                        result.extend(entities.iter().copied());
                    }
                }
            }
        }

        result
    }

    /// Get potential collision pairs (broadphase)
    pub fn get_potential_pairs(&self) -> Vec<(Entity, Entity)> {
        let mut pairs = Vec::new();

        for entities in self.cells.values() {
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    pairs.push((entities[i], entities[j]));
                }
            }
        }

        // Remove duplicates (entities in multiple cells)
        pairs.sort();
        pairs.dedup();
        pairs
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(10.0)
    }
}
