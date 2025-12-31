//! Editor application state and initialization.

use std::sync::Arc;
use glam::Vec3;
use winit::window::Window as WinitWindow;

use crate::render::{RenderContext, IsometricCamera, EguiIntegration};
use crate::editor::viewport::Viewport;
use crate::editor::selection::{SceneObject, Selection};
use crate::editor::panels::{HierarchyPanel, ToolbarPanel, InspectorPanel, AssetBrowser};
use crate::editor::gizmos::Gizmo;
use crate::editor::commands::CommandHistory;
use crate::editor::shortcuts::ShortcutManager;
use crate::editor::scene::SceneManager;
use crate::editor::snap::SnapSettings;
use crate::editor::prefab::Prefab;
#[cfg(feature = "scripting")]
use crate::scripting::{ScriptRuntime, ScriptContext};

/// File dialog action type
#[derive(Clone, Copy, Debug)]
pub enum FileDialogAction {
    Open,
    #[allow(dead_code)]
    Save,
    SaveAs,
}

/// Input modifiers state
#[derive(Clone, Copy, Debug, Default)]
pub struct InputModifiers {
    pub ctrl: bool,
    pub shift: bool,
    #[allow(dead_code)]
    pub alt: bool,
}

/// Editor application state
pub struct EditorApp {
    /// Render context
    pub(crate) ctx: Option<RenderContext>,
    /// Egui integration
    pub(crate) egui: Option<EguiIntegration>,
    /// Window reference for egui
    pub(crate) window: Option<Arc<WinitWindow>>,
    /// 3D Viewport
    pub(crate) viewport: Option<Viewport>,
    /// Camera
    pub(crate) camera: IsometricCamera,
    /// Frame counter
    pub(crate) frame_count: u64,
    /// Scene objects
    pub(crate) scene_objects: Vec<SceneObject>,
    /// Selection state
    pub(crate) selection: Selection,
    /// Next object ID
    pub(crate) next_id: u32,
    /// Hierarchy panel
    pub(crate) hierarchy_panel: HierarchyPanel,
    /// Toolbar panel
    pub(crate) toolbar_panel: ToolbarPanel,
    /// Inspector panel
    pub(crate) inspector_panel: InspectorPanel,
    /// Last viewport size (for resize detection)
    pub(crate) last_viewport_size: (f32, f32),
    /// Viewport rect for mouse picking
    pub(crate) viewport_rect: egui::Rect,
    /// Mouse position in viewport
    pub(crate) mouse_in_viewport: Option<egui::Pos2>,
    /// Gizmo for manipulation
    pub(crate) gizmo: Gizmo,
    /// Command history for undo/redo
    pub(crate) command_history: CommandHistory,
    /// Keyboard shortcuts manager
    pub(crate) shortcuts: ShortcutManager,
    /// Scene manager
    pub(crate) scene_manager: SceneManager,
    /// File dialog state
    pub(crate) file_dialog_action: Option<FileDialogAction>,
    /// File path input for dialogs
    pub(crate) file_path_input: String,
    /// Input modifiers (Ctrl, Shift, Alt)
    pub(crate) input_modifiers: InputModifiers,
    /// Snap settings for transform operations
    pub(crate) snap_settings: SnapSettings,
    /// Clipboard for copy/paste operations
    pub(crate) clipboard: Vec<SceneObject>,
    /// Saved prefabs
    pub(crate) prefabs: Vec<Prefab>,
    /// Prefab name input for dialog
    pub(crate) prefab_name_input: String,
    /// Whether prefab dialog is open
    pub(crate) show_prefab_dialog: bool,
    /// Asset browser panel
    pub(crate) asset_browser: AssetBrowser,
    /// Script runtime (only with scripting feature)
    #[cfg(feature = "scripting")]
    pub(crate) script_runtime: ScriptRuntime,
    /// Script context for passing data to scripts
    #[cfg(feature = "scripting")]
    pub(crate) script_context: ScriptContext,
    /// Whether script dialog is open
    pub(crate) show_script_dialog: bool,
    /// Script path input for dialog
    pub(crate) script_path_input: String,
    /// Whether to open script in editor after creation
    pub(crate) open_script_in_editor: bool,
    /// Whether play mode is active
    pub(crate) is_playing: bool,
    /// Saved scene state for reset when stopping
    pub(crate) saved_scene_state: Vec<SceneObject>,
    /// Elapsed time in play mode
    pub(crate) play_time: f32,
    /// Last frame instant for delta calculation
    pub(crate) last_frame_instant: Option<std::time::Instant>,
    /// Whether to show splash screen
    pub(crate) show_splash: bool,
    /// When splash started
    pub(crate) splash_start_time: Option<std::time::Instant>,
    /// Splash texture handle
    pub(crate) splash_texture: Option<egui::TextureHandle>,
    /// Current project
    pub(crate) current_project: Option<crate::editor::project::Project>,
    /// Show project properties dialog
    pub(crate) show_project_dialog: bool,
}

impl EditorApp {
    /// Create a new editor application
    pub fn new() -> Self {
        // Create initial scene objects
        let mut scene_objects = Vec::new();
        let mut next_id = 0u32;

        // Add some initial cubes
        let positions = [
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(2.0, 0.5, 0.0),
            Vec3::new(-2.0, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 2.0),
            Vec3::new(0.0, 0.5, -2.0),
        ];

        for pos in positions {
            scene_objects.push(SceneObject::cube(next_id, pos));
            next_id += 1;
        }

        Self {
            ctx: None,
            egui: None,
            window: None,
            viewport: None,
            camera: IsometricCamera::default(),
            frame_count: 0,
            scene_objects,
            selection: Selection::new(),
            next_id,
            hierarchy_panel: HierarchyPanel::new(),
            toolbar_panel: ToolbarPanel::new(),
            inspector_panel: InspectorPanel::new(),
            last_viewport_size: (0.0, 0.0),
            viewport_rect: egui::Rect::NOTHING,
            mouse_in_viewport: None,
            gizmo: Gizmo::new(),
            command_history: CommandHistory::new(),
            shortcuts: ShortcutManager::new(),
            scene_manager: SceneManager::new(),
            file_dialog_action: None,
            file_path_input: String::new(),
            input_modifiers: InputModifiers::default(),
            snap_settings: SnapSettings::new(),
            clipboard: Vec::new(),
            prefabs: Vec::new(),
            prefab_name_input: String::new(),
            show_prefab_dialog: false,
            asset_browser: AssetBrowser::new(std::path::PathBuf::from(".")),
            #[cfg(feature = "scripting")]
            script_runtime: ScriptRuntime::new(),
            #[cfg(feature = "scripting")]
            script_context: ScriptContext::new(),
            show_script_dialog: false,
            script_path_input: String::new(),
            open_script_in_editor: true,
            is_playing: false,
            saved_scene_state: Vec::new(),
            play_time: 0.0,
            last_frame_instant: None,
            show_splash: false,
            splash_start_time: None,
            splash_texture: None,
            current_project: None,
            show_project_dialog: false,
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}
