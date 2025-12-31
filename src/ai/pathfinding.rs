//! # A* Pathfinding

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use glam::Vec3;

/// Grid node for pathfinding
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathNode {
    pub x: i32,
    pub y: i32,
}

impl PathNode {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_world(&self, grid: &Grid) -> Vec3 {
        Vec3::new(
            self.x as f32 * grid.cell_size + grid.cell_size / 2.0,
            0.0,
            self.y as f32 * grid.cell_size + grid.cell_size / 2.0,
        )
    }

    pub fn from_world(pos: Vec3, grid: &Grid) -> Self {
        Self {
            x: (pos.x / grid.cell_size).floor() as i32,
            y: (pos.z / grid.cell_size).floor() as i32,
        }
    }

    fn neighbors(&self) -> [PathNode; 8] {
        [
            PathNode::new(self.x - 1, self.y),
            PathNode::new(self.x + 1, self.y),
            PathNode::new(self.x, self.y - 1),
            PathNode::new(self.x, self.y + 1),
            PathNode::new(self.x - 1, self.y - 1),
            PathNode::new(self.x + 1, self.y - 1),
            PathNode::new(self.x - 1, self.y + 1),
            PathNode::new(self.x + 1, self.y + 1),
        ]
    }

    fn heuristic(&self, goal: PathNode) -> f32 {
        let dx = (self.x - goal.x).abs() as f32;
        let dy = (self.y - goal.y).abs() as f32;
        // Diagonal distance heuristic
        dx.max(dy) + (2.0_f32.sqrt() - 1.0) * dx.min(dy)
    }
}

/// Navigation grid
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub cell_size: f32,
    blocked: Vec<bool>,
}

impl Grid {
    pub fn new(width: i32, height: i32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            blocked: vec![false; (width * height) as usize],
        }
    }

    pub fn is_blocked(&self, node: PathNode) -> bool {
        if node.x < 0 || node.x >= self.width || node.y < 0 || node.y >= self.height {
            return true;
        }
        self.blocked[(node.y * self.width + node.x) as usize]
    }

    pub fn set_blocked(&mut self, node: PathNode, blocked: bool) {
        if node.x >= 0 && node.x < self.width && node.y >= 0 && node.y < self.height {
            self.blocked[(node.y * self.width + node.x) as usize] = blocked;
        }
    }

    pub fn is_walkable(&self, node: PathNode) -> bool {
        !self.is_blocked(node)
    }
}

/// Path result
#[derive(Clone, Debug)]
pub struct Path {
    pub nodes: Vec<PathNode>,
    pub cost: f32,
}

impl Path {
    pub fn empty() -> Self {
        Self { nodes: Vec::new(), cost: 0.0 }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn to_world_points(&self, grid: &Grid) -> Vec<Vec3> {
        self.nodes.iter().map(|n| n.to_world(grid)).collect()
    }
}

// A* internal node
#[derive(Clone, Copy)]
struct AStarNode {
    node: PathNode,
    f_score: f32,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.partial_cmp(&self.f_score).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* pathfinder
pub struct AStar;

impl AStar {
    /// Find path from start to goal
    pub fn find_path(grid: &Grid, start: PathNode, goal: PathNode) -> Option<Path> {
        if grid.is_blocked(start) || grid.is_blocked(goal) {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<PathNode, PathNode> = HashMap::new();
        let mut g_score: HashMap<PathNode, f32> = HashMap::new();

        g_score.insert(start, 0.0);
        open_set.push(AStarNode {
            node: start,
            f_score: start.heuristic(goal),
        });

        while let Some(current) = open_set.pop() {
            if current.node == goal {
                // Reconstruct path
                let mut path = vec![goal];
                let mut node = goal;
                while let Some(&prev) = came_from.get(&node) {
                    path.push(prev);
                    node = prev;
                }
                path.reverse();
                return Some(Path {
                    cost: g_score[&goal],
                    nodes: path,
                });
            }

            for neighbor in current.node.neighbors() {
                if grid.is_blocked(neighbor) {
                    continue;
                }

                // Diagonal movement cost
                let is_diagonal = neighbor.x != current.node.x && neighbor.y != current.node.y;
                let move_cost = if is_diagonal { 2.0_f32.sqrt() } else { 1.0 };

                let tentative_g = g_score.get(&current.node).unwrap_or(&f32::MAX) + move_cost;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f32::MAX) {
                    came_from.insert(neighbor, current.node);
                    g_score.insert(neighbor, tentative_g);
                    open_set.push(AStarNode {
                        node: neighbor,
                        f_score: tentative_g + neighbor.heuristic(goal),
                    });
                }
            }
        }

        None // No path found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let grid = Grid::new(10, 10, 1.0);
        let start = PathNode::new(0, 0);
        let goal = PathNode::new(5, 5);

        let path = AStar::find_path(&grid, start, goal);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.nodes.first(), Some(&start));
        assert_eq!(path.nodes.last(), Some(&goal));
    }

    #[test]
    fn test_blocked_path() {
        let mut grid = Grid::new(5, 5, 1.0);
        // Block middle row
        for x in 0..5 {
            grid.set_blocked(PathNode::new(x, 2), true);
        }

        let start = PathNode::new(2, 0);
        let goal = PathNode::new(2, 4);

        let path = AStar::find_path(&grid, start, goal);
        assert!(path.is_none());
    }
}
