# Process: FPS Display, VSync, Auto-Save Settings, and Modularization

**Date:** 2025-12-31

## Summary

Implemented FPS counter display, VSync option, auto-save for project settings, and modularized dialog files.

## Changes Made

### 1. FPS Display in GameWindow

- Added egui integration to `game_window.rs` for rendering FPS overlay
- FPS counter displays with yellow text (#FFDC00) and black outline for visibility
- FPS is smoothed over 30 samples to avoid flickering
- Only shows when `show_fps` is enabled in project settings

**Files modified:**
- `src/editor/game_window.rs` - Added egui, FPS tracking, and overlay rendering

### 2. VSync Option

- Added `vsync` field to `ProjectConfig` with default value `true`
- Added checkbox in Window tab of project properties dialog
- Created `RenderContext::new_with_vsync()` method that uses:
  - `PresentMode::AutoVsync` when vsync is enabled
  - `PresentMode::Immediate` or `PresentMode::Mailbox` when disabled
- Updated `GameSettings` to include `vsync` field
- Updated `GameWindow::new()` to use vsync setting when creating RenderContext

**Files modified:**
- `src/editor/project.rs` - Added vsync field
- `src/render/context.rs` - Added new_with_vsync() method
- `src/editor/game_window.rs` - Added vsync to GameSettings, use new_with_vsync()
- `src/editor/app/lifecycle.rs` - Pass vsync to GameSettings
- `src/editor/app/dialogs/project_dialog.rs` - Added VSync checkbox

### 3. Auto-Save Project Settings

- Modified project properties dialog to auto-save when closed
- Settings are saved when clicking "Close" or pressing the X button
- Removed separate "Cancel" and "Save" buttons, replaced with single "Close"

**Files modified:**
- `src/editor/app/dialogs/project_dialog.rs`

### 4. GameSettings Integration

- Updated `lifecycle.rs` to build `GameSettings` from project config
- Passes settings to `GameWindow::new()` including:
  - window_width, window_height
  - splash_duration
  - background_color
  - show_fps
  - vsync

**Files modified:**
- `src/editor/app/lifecycle.rs`

### 5. Modularization: Dialogs

Split `dialogs.rs` (575 lines) into separate modules:

| File | Description | Lines |
|------|-------------|-------|
| `dialogs/mod.rs` | Module declaration | 12 |
| `dialogs/file_dialog.rs` | Open/Save scene dialogs | 45 |
| `dialogs/prefab_dialog.rs` | Prefab creation dialog | 55 |
| `dialogs/script_dialog.rs` | Script create/attach dialogs | 285 |
| `dialogs/project_dialog.rs` | Project properties dialog | 165 |

**Files created:**
- `src/editor/app/dialogs/mod.rs`
- `src/editor/app/dialogs/file_dialog.rs`
- `src/editor/app/dialogs/prefab_dialog.rs`
- `src/editor/app/dialogs/script_dialog.rs`
- `src/editor/app/dialogs/project_dialog.rs`

**Files deleted:**
- `src/editor/app/dialogs.rs`

## Audit Results

Files checked for >500 lines:

| File | Lines | Status |
|------|-------|--------|
| dialogs.rs | 575 | Split into modules |
| game_window.rs | ~556 | Kept as-is (cohesive) |
| lifecycle.rs | ~450 | OK |
| ui.rs | 387 | OK |
| api.rs | 391 | OK |
| menu.rs | 370 | OK |

## Build Status

All changes compile successfully with only minor dead_code warning for `is_in_splash()` method.
