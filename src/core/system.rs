//! # System Scheduler
//!
//! Systems are functions that operate on components. The scheduler
//! manages system execution order and parallelization.
//!
//! ## Usage
//!
//! ```rust,ignore
//! fn movement_system(world: &mut World) {
//!     for (pos, vel) in query_two_mut::<Position, Velocity>(world) {
//!         pos.x += vel.dx;
//!         pos.y += vel.dy;
//!     }
//! }
//!
//! let mut scheduler = SystemScheduler::new();
//! scheduler.add_system(Stage::Update, movement_system);
//! scheduler.run(&mut world);
//! ```

use std::collections::HashMap;

use super::world::World;

/// Execution stage for organizing system order
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    /// First stage - input processing
    PreUpdate,
    /// Main game logic
    Update,
    /// After main logic - physics, AI
    PostUpdate,
    /// Final stage - rendering prep
    PreRender,
    /// Rendering
    Render,
    /// Cleanup
    PostRender,
}

impl Stage {
    /// Get all stages in execution order
    pub fn all() -> &'static [Stage] {
        &[
            Stage::PreUpdate,
            Stage::Update,
            Stage::PostUpdate,
            Stage::PreRender,
            Stage::Render,
            Stage::PostRender,
        ]
    }
}

/// A system is a function that operates on the world
pub trait System: Send + Sync {
    /// Run the system
    fn run(&self, world: &mut World);

    /// System name for debugging
    fn name(&self) -> &str {
        "unnamed_system"
    }
}

/// Wrapper for function pointers as systems
struct FnSystem {
    name: String,
    func: fn(&mut World),
}

impl System for FnSystem {
    fn run(&self, world: &mut World) {
        (self.func)(world);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Wrapper for boxed closures as systems
struct BoxedSystem {
    name: String,
    func: Box<dyn Fn(&mut World) + Send + Sync>,
}

impl System for BoxedSystem {
    fn run(&self, world: &mut World) {
        (self.func)(world);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Manages system registration and execution
pub struct SystemScheduler {
    /// Systems organized by stage
    stages: HashMap<Stage, Vec<Box<dyn System>>>,
    /// Whether the scheduler is running
    running: bool,
}

impl SystemScheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        let mut stages = HashMap::new();
        for stage in Stage::all() {
            stages.insert(*stage, Vec::new());
        }
        Self {
            stages,
            running: false,
        }
    }

    /// Add a function as a system
    pub fn add_system(&mut self, stage: Stage, system: fn(&mut World)) -> &mut Self {
        let name = format!("fn_system_{}", self.stages[&stage].len());
        self.stages.get_mut(&stage).unwrap().push(Box::new(FnSystem {
            name,
            func: system,
        }));
        self
    }

    /// Add a named function as a system
    pub fn add_system_named(
        &mut self,
        stage: Stage,
        name: impl Into<String>,
        system: fn(&mut World),
    ) -> &mut Self {
        self.stages.get_mut(&stage).unwrap().push(Box::new(FnSystem {
            name: name.into(),
            func: system,
        }));
        self
    }

    /// Add a closure as a system
    pub fn add_system_boxed<F>(&mut self, stage: Stage, name: impl Into<String>, system: F) -> &mut Self
    where
        F: Fn(&mut World) + Send + Sync + 'static,
    {
        self.stages.get_mut(&stage).unwrap().push(Box::new(BoxedSystem {
            name: name.into(),
            func: Box::new(system),
        }));
        self
    }

    /// Add a trait object system
    pub fn add_system_dyn(&mut self, stage: Stage, system: Box<dyn System>) -> &mut Self {
        self.stages.get_mut(&stage).unwrap().push(system);
        self
    }

    /// Run all systems for one frame
    pub fn run(&mut self, world: &mut World) {
        self.running = true;

        for stage in Stage::all() {
            self.run_stage(*stage, world);
        }

        self.running = false;
    }

    /// Run systems for a specific stage
    pub fn run_stage(&mut self, stage: Stage, world: &mut World) {
        if let Some(systems) = self.stages.get(&stage) {
            for system in systems {
                system.run(world);
            }
        }
    }

    /// Check if scheduler is currently running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get number of systems in a stage
    pub fn system_count(&self, stage: Stage) -> usize {
        self.stages.get(&stage).map(|s| s.len()).unwrap_or(0)
    }

    /// Get total number of systems
    pub fn total_systems(&self) -> usize {
        self.stages.values().map(|s| s.len()).sum()
    }

    /// Clear all systems
    pub fn clear(&mut self) {
        for systems in self.stages.values_mut() {
            systems.clear();
        }
    }

    /// List system names (for debugging)
    pub fn list_systems(&self) -> Vec<(&Stage, Vec<&str>)> {
        Stage::all()
            .iter()
            .map(|stage| {
                let names: Vec<&str> = self.stages[stage]
                    .iter()
                    .map(|s| s.name())
                    .collect();
                (stage, names)
            })
            .collect()
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple system set for organizing related systems
pub struct SystemSet {
    name: String,
    systems: Vec<(Stage, Box<dyn System>)>,
}

impl SystemSet {
    /// Create a new system set
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
        }
    }

    /// Add a system to this set
    pub fn add(mut self, stage: Stage, system: fn(&mut World)) -> Self {
        let name = format!("{}::{}", self.name, self.systems.len());
        self.systems.push((stage, Box::new(FnSystem { name, func: system })));
        self
    }

    /// Register all systems from this set into a scheduler
    pub fn register(self, scheduler: &mut SystemScheduler) {
        for (stage, system) in self.systems {
            scheduler.add_system_dyn(stage, system);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_scheduler_basic() {
        let mut world = World::new();
        let mut scheduler = SystemScheduler::new();

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        fn test_system(_world: &mut World) {
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        scheduler.add_system(Stage::Update, test_system);
        scheduler.run(&mut world);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_stage_order() {
        let mut world = World::new();
        let mut scheduler = SystemScheduler::new();

        static ORDER: AtomicU32 = AtomicU32::new(0);

        fn pre_update_system(_: &mut World) {
            assert_eq!(ORDER.fetch_add(1, Ordering::SeqCst), 0);
        }

        fn update_system(_: &mut World) {
            assert_eq!(ORDER.fetch_add(1, Ordering::SeqCst), 1);
        }

        fn post_update_system(_: &mut World) {
            assert_eq!(ORDER.fetch_add(1, Ordering::SeqCst), 2);
        }

        scheduler.add_system(Stage::PreUpdate, pre_update_system);
        scheduler.add_system(Stage::Update, update_system);
        scheduler.add_system(Stage::PostUpdate, post_update_system);

        scheduler.run(&mut world);
    }

    #[test]
    fn test_system_set() {
        let mut scheduler = SystemScheduler::new();

        fn sys1(_: &mut World) {}
        fn sys2(_: &mut World) {}

        let set = SystemSet::new("test_set")
            .add(Stage::Update, sys1)
            .add(Stage::PostUpdate, sys2);

        set.register(&mut scheduler);

        assert_eq!(scheduler.total_systems(), 2);
    }
}
