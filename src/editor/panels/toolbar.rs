//! # Toolbar Panel
//!
//! Main toolbar with tools and quick actions.

use egui::Ui;

/// Current editor tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    /// Selection tool (default)
    #[default]
    Select,
    /// Move/translate tool
    Move,
    /// Rotate tool
    Rotate,
    /// Scale tool
    Scale,
}

impl Tool {
    /// Get the tool icon
    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Select => "⬚", // Box
            Tool::Move => "✥",   // Cross arrows
            Tool::Rotate => "↻", // Rotate
            Tool::Scale => "⤢",  // Resize
        }
    }

    /// Get the tool name
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Move => "Move",
            Tool::Rotate => "Rotate",
            Tool::Scale => "Scale",
        }
    }

    /// Get the keyboard shortcut
    pub fn shortcut(&self) -> &'static str {
        match self {
            Tool::Select => "Q",
            Tool::Move => "W",
            Tool::Rotate => "E",
            Tool::Scale => "R",
        }
    }
}

/// Transform space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformSpace {
    /// Local/object space
    #[default]
    Local,
    /// World/global space
    World,
}

/// Toolbar panel
pub struct ToolbarPanel {
    /// Current tool
    pub current_tool: Tool,
    /// Transform space
    pub transform_space: TransformSpace,
    /// Snap enabled
    pub snap_enabled: bool,
    /// Snap value for translation
    pub snap_translate: f32,
    /// Snap value for rotation (degrees)
    pub snap_rotate: f32,
    /// Snap value for scale
    pub snap_scale: f32,
}

impl Default for ToolbarPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolbarPanel {
    /// Create a new toolbar
    pub fn new() -> Self {
        Self {
            current_tool: Tool::Select,
            transform_space: TransformSpace::Local,
            snap_enabled: false,
            snap_translate: 1.0,
            snap_rotate: 15.0,
            snap_scale: 0.1,
        }
    }

    /// Draw the toolbar (horizontal)
    pub fn show(&mut self, ui: &mut Ui) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal(|ui| {
            // Tool buttons
            ui.label("Tools:");

            for tool in [Tool::Select, Tool::Move, Tool::Rotate, Tool::Scale] {
                let is_selected = self.current_tool == tool;
                let text = format!("{} {}", tool.icon(), tool.shortcut());

                if ui
                    .selectable_label(is_selected, text)
                    .on_hover_text(tool.name())
                    .clicked()
                {
                    self.current_tool = tool;
                }
            }

            ui.separator();

            // Transform space toggle
            let space_text = match self.transform_space {
                TransformSpace::Local => "Local",
                TransformSpace::World => "World",
            };
            if ui.button(space_text).clicked() {
                self.transform_space = match self.transform_space {
                    TransformSpace::Local => TransformSpace::World,
                    TransformSpace::World => TransformSpace::Local,
                };
            }

            ui.separator();

            // Snap toggle
            ui.checkbox(&mut self.snap_enabled, "Snap");

            if self.snap_enabled {
                ui.label("|");
                match self.current_tool {
                    Tool::Move => {
                        ui.add(
                            egui::DragValue::new(&mut self.snap_translate)
                                .speed(0.1)
                                .range(0.1..=10.0)
                                .prefix("T: ")
                                .fixed_decimals(1),
                        );
                    }
                    Tool::Rotate => {
                        ui.add(
                            egui::DragValue::new(&mut self.snap_rotate)
                                .speed(1.0)
                                .range(1.0..=90.0)
                                .suffix("°")
                                .prefix("R: ")
                                .fixed_decimals(0),
                        );
                    }
                    Tool::Scale => {
                        ui.add(
                            egui::DragValue::new(&mut self.snap_scale)
                                .speed(0.01)
                                .range(0.01..=1.0)
                                .prefix("S: ")
                                .fixed_decimals(2),
                        );
                    }
                    _ => {}
                }
            }

            ui.separator();

            // Quick actions
            if ui
                .button("Center")
                .on_hover_text("Center view on selection")
                .clicked()
            {
                action = ToolbarAction::CenterView;
            }

            if ui
                .button("Frame All")
                .on_hover_text("Frame all objects")
                .clicked()
            {
                action = ToolbarAction::FrameAll;
            }
        });

        action
    }

    /// Handle keyboard shortcuts
    pub fn handle_key(&mut self, key: egui::Key) -> bool {
        match key {
            egui::Key::Q => {
                self.current_tool = Tool::Select;
                true
            }
            egui::Key::W => {
                self.current_tool = Tool::Move;
                true
            }
            egui::Key::E => {
                self.current_tool = Tool::Rotate;
                true
            }
            egui::Key::R => {
                self.current_tool = Tool::Scale;
                true
            }
            _ => false,
        }
    }
}

/// Actions from the toolbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// No action
    None,
    /// Center view on selection
    CenterView,
    /// Frame all objects in view
    FrameAll,
}
