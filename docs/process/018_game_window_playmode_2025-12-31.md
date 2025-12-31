# 018 - Game Window Play Mode

**Date:** 2025-12-31

## Summary

Implemented a separate game window for play mode, replacing the previous in-editor play mode with splash screen. Also changed project file extension from `.ron` to `.xtrm`.

## Changes Made

### 1. Game Window Module (`src/editor/game_window.rs`)

Created a new module that provides a separate window for running the game:

- `GameWindow` struct with its own `RenderContext`, camera, and rendering pipeline
- Complete wgpu rendering setup (pipeline, bind groups, buffers)
- Scene object rendering with uniform buffers
- Simple rotation animation during play mode
- Window resize handling
- ESC key to close game window

### 2. App Trait Extension (`src/render/window.rs`)

Extended the `App` trait to support multiple windows:

- `handle_pending_windows(&mut self, event_loop: &ActiveEventLoop)` - Called each frame to allow apps to create secondary windows
- `secondary_window_event(...)` - Handle events for secondary windows
- `main_window_id(&self)` - Get the main window ID

### 3. EditorApp Integration (`src/editor/app/lifecycle.rs`)

Implemented the new App trait methods:

- `handle_pending_windows` - Creates the game window when `pending_game_start` is true
- `secondary_window_event` - Handles game window events (resize, redraw, close, ESC key)
- `main_window_id` - Returns the editor window ID

### 4. Play Mode Updates (`src/editor/app/play_mode.rs`)

Simplified play mode to use game window:

- `start_play()` - Sets `pending_game_start = true` (window created in next frame)
- `stop_play()` - Calls `stop_play_and_close_game_window()`
- `toggle_play()` - Toggles between play/stop states
- Removed splash screen related code

### 5. EditorApp State (`src/editor/app/state.rs`)

Added new fields:

- `game_window: Option<GameWindow>` - The game window instance
- `pending_game_start: bool` - Flag to request game window creation

### 6. Project Extension (`src/editor/project.rs`)

Changed project file extension from `.ron` to `.xtrm`:

- `project.xtrm` is now the project configuration file name
- Scene files still use `.ron` format

### 7. Camera Clone (`src/render/camera.rs`)

Added `#[derive(Clone)]` to `IsometricCamera` to allow copying camera state to game window.

## Technical Notes

### Multi-Window Architecture

The winit event loop now routes events based on window ID:
1. Main window events go to the normal App trait methods
2. Secondary window events go to `secondary_window_event()`

### Window Creation Timing

Windows can only be created when we have access to `ActiveEventLoop`. The `pending_game_start` flag allows the EditorApp to request window creation, which is fulfilled in `about_to_wait()` callback.

### Rendering

The game window has its own complete rendering setup:
- Separate `RenderContext` with surface
- Own depth buffer
- Mesh rendering pipeline with basic.wgsl shader
- Independent frame rendering

## Files Modified

- `src/render/window.rs` - Extended App trait for multi-window
- `src/render/camera.rs` - Added Clone derive
- `src/editor/mod.rs` - Added game_window module export
- `src/editor/game_window.rs` - **NEW** Complete game window implementation
- `src/editor/app/state.rs` - Added game_window and pending_game_start fields
- `src/editor/app/lifecycle.rs` - Implemented multi-window event handling
- `src/editor/app/play_mode.rs` - Simplified to use game window
- `src/editor/app/splash.rs` - Deprecated splash screen code
- `src/editor/project.rs` - Changed extension to .xtrm

## Testing

- Compile check passed with `cargo check`
- Compile check passed with `cargo check --features scripting`
- No warnings after fixes
