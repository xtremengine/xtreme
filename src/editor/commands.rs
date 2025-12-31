//! # Command System
//!
//! Undo/Redo functionality using the Command Pattern.

use glam::Vec3;

/// A command that can be executed, undone, and redone
#[derive(Clone, Debug)]
pub enum Command {
    /// Transform an object (position, rotation, scale)
    Transform {
        object_id: u32,
        old_position: Vec3,
        old_rotation: Vec3,
        old_scale: Vec3,
        new_position: Vec3,
        new_rotation: Vec3,
        new_scale: Vec3,
    },
    /// Transform multiple objects at once (for multi-selection)
    BatchTransform {
        object_ids: Vec<u32>,
        old_transforms: Vec<(Vec3, Vec3, Vec3)>,
        new_transforms: Vec<(Vec3, Vec3, Vec3)>,
    },
    /// Create an object
    Create {
        object_id: u32,
        name: String,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: [f32; 4],
        visible: bool,
    },
    /// Delete an object
    Delete {
        object_id: u32,
        name: String,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: [f32; 4],
        visible: bool,
    },
    /// Rename an object
    Rename {
        object_id: u32,
        old_name: String,
        new_name: String,
    },
    /// Change object color
    ChangeColor {
        object_id: u32,
        old_color: [f32; 4],
        new_color: [f32; 4],
    },
    /// Toggle visibility
    ToggleVisibility { object_id: u32, was_visible: bool },
    /// Batch of commands (for complex operations)
    Batch(Vec<Command>),
}

impl Command {
    /// Get a description of this command
    pub fn description(&self) -> String {
        match self {
            Command::Transform { object_id, .. } => format!("Transform object {}", object_id),
            Command::BatchTransform { object_ids, .. } => {
                if object_ids.len() == 1 {
                    format!("Transform object {}", object_ids[0])
                } else {
                    format!("Transform {} objects", object_ids.len())
                }
            }
            Command::Create { name, .. } => format!("Create {}", name),
            Command::Delete { name, .. } => format!("Delete {}", name),
            Command::Rename {
                old_name, new_name, ..
            } => format!("Rename {} to {}", old_name, new_name),
            Command::ChangeColor { object_id, .. } => {
                format!("Change color of object {}", object_id)
            }
            Command::ToggleVisibility { object_id, .. } => {
                format!("Toggle visibility of object {}", object_id)
            }
            Command::Batch(cmds) => format!("Batch ({} commands)", cmds.len()),
        }
    }
}

/// Command history for undo/redo
pub struct CommandHistory {
    /// Commands that have been executed (can be undone)
    undo_stack: Vec<Command>,
    /// Commands that have been undone (can be redone)
    redo_stack: Vec<Command>,
    /// Maximum history size
    max_size: usize,
    /// Whether we're currently tracking a drag operation
    tracking_drag: bool,
    /// The command being built during drag (single object)
    pending_transform: Option<Command>,
    /// Pending batch transform (multi-selection)
    pending_batch: Option<Command>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    /// Create a new command history
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 100,
            tracking_drag: false,
            pending_transform: None,
            pending_batch: None,
        }
    }

    /// Execute a command and add it to history
    pub fn execute(&mut self, command: Command) {
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Clear redo stack on new command

        // Limit history size
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Start tracking a drag operation (for transform commands)
    pub fn begin_drag(&mut self, object_id: u32, position: Vec3, rotation: Vec3, scale: Vec3) {
        self.tracking_drag = true;
        self.pending_transform = Some(Command::Transform {
            object_id,
            old_position: position,
            old_rotation: rotation,
            old_scale: scale,
            new_position: position,
            new_rotation: rotation,
            new_scale: scale,
        });
    }

    /// Update the pending transform during drag
    pub fn update_drag(&mut self, position: Vec3, rotation: Vec3, scale: Vec3) {
        if let Some(Command::Transform {
            new_position,
            new_rotation,
            new_scale,
            ..
        }) = &mut self.pending_transform
        {
            *new_position = position;
            *new_rotation = rotation;
            *new_scale = scale;
        }
    }

    /// End drag and commit the transform command
    pub fn end_drag(&mut self) {
        self.tracking_drag = false;
        if let Some(cmd) = self.pending_transform.take() {
            // Only add if there was actual change
            if let Command::Transform {
                old_position,
                old_rotation,
                old_scale,
                new_position,
                new_rotation,
                new_scale,
                ..
            } = &cmd
            {
                if *old_position != *new_position
                    || *old_rotation != *new_rotation
                    || *old_scale != *new_scale
                {
                    self.execute(cmd);
                }
            }
        }
    }

    /// Start tracking a batch drag operation (for multi-selection)
    pub fn begin_drag_batch(&mut self, object_ids: &[u32], transforms: Vec<(Vec3, Vec3, Vec3)>) {
        self.tracking_drag = true;
        self.pending_batch = Some(Command::BatchTransform {
            object_ids: object_ids.to_vec(),
            old_transforms: transforms.clone(),
            new_transforms: transforms,
        });
    }

    /// Update the pending batch transform during drag
    pub fn update_drag_batch(&mut self, transforms: Vec<(Vec3, Vec3, Vec3)>) {
        if let Some(Command::BatchTransform { new_transforms, .. }) = &mut self.pending_batch {
            *new_transforms = transforms;
        }
    }

    /// End batch drag and commit the transform command
    pub fn end_drag_batch(&mut self) {
        self.tracking_drag = false;
        if let Some(cmd) = self.pending_batch.take() {
            // Only add if there was actual change
            if let Command::BatchTransform {
                old_transforms,
                new_transforms,
                ..
            } = &cmd
            {
                let has_changes = old_transforms
                    .iter()
                    .zip(new_transforms.iter())
                    .any(|(old, new)| old.0 != new.0 || old.1 != new.1 || old.2 != new.2);
                if has_changes {
                    self.execute(cmd);
                }
            }
        }
    }

    /// Cancel current drag without committing
    pub fn cancel_drag(&mut self) {
        self.tracking_drag = false;
        self.pending_transform = None;
        self.pending_batch = None;
    }

    /// Check if currently tracking a drag
    pub fn is_tracking_drag(&self) -> bool {
        self.tracking_drag
    }

    /// Get the pending transform for potential rollback
    pub fn pending_transform(&self) -> Option<&Command> {
        self.pending_transform.as_ref()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the next undo command (without removing it)
    pub fn peek_undo(&self) -> Option<&Command> {
        self.undo_stack.last()
    }

    /// Get the next redo command (without removing it)
    pub fn peek_redo(&self) -> Option<&Command> {
        self.redo_stack.last()
    }

    /// Pop and return the command to undo
    pub fn undo(&mut self) -> Option<Command> {
        if let Some(cmd) = self.undo_stack.pop() {
            self.redo_stack.push(cmd.clone());
            Some(cmd)
        } else {
            None
        }
    }

    /// Pop and return the command to redo
    pub fn redo(&mut self) -> Option<Command> {
        if let Some(cmd) = self.redo_stack.pop() {
            self.undo_stack.push(cmd.clone());
            Some(cmd)
        } else {
            None
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_transform = None;
        self.tracking_drag = false;
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut history = CommandHistory::new();

        let cmd = Command::Transform {
            object_id: 1,
            old_position: Vec3::ZERO,
            old_rotation: Vec3::ZERO,
            old_scale: Vec3::ONE,
            new_position: Vec3::new(1.0, 0.0, 0.0),
            new_rotation: Vec3::ZERO,
            new_scale: Vec3::ONE,
        };

        history.execute(cmd);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redone = history.redo();
        assert!(redone.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_drag_tracking() {
        let mut history = CommandHistory::new();

        history.begin_drag(1, Vec3::ZERO, Vec3::ZERO, Vec3::ONE);
        assert!(history.is_tracking_drag());

        history.update_drag(Vec3::new(5.0, 0.0, 0.0), Vec3::ZERO, Vec3::ONE);
        history.end_drag();

        assert!(!history.is_tracking_drag());
        assert!(history.can_undo());
    }
}
