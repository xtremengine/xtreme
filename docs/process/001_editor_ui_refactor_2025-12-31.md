# Process: Editor UI Refactoring

**Date:** 2025-12-31
**Type:** Refactoring
**Status:** Completed (Phase 1 & 2)

## Objective

Reduce editor files from >500 lines to under 500 lines by extracting functionality into separate modules.

## Phase 1: UI Refactoring

### New Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `src/editor/app/templates.rs` | Shader/Script/Scene templates | 85 |
| `src/editor/panels/camera_settings.rs` | Camera controls UI | 62 |
| `src/editor/panels/snap_settings.rs` | Snap settings UI | 55 |
| `src/editor/panels/scripts.rs` | Scripts section UI | 83 |

### Results

- **Before:** `ui.rs` had 698 lines
- **After:** `ui.rs` has 486 lines
- **Reduction:** 212 lines (30%)

## Phase 2: Viewport & Inspector Refactoring

### Viewport Changes

| File | Before | After |
|------|--------|-------|
| `viewport/mod.rs` | 682 | 271 |
| `viewport/pipelines.rs` | - | 479 (new) |

### Inspector Changes

Converted `inspector.rs` (558 lines) to submodule:

| File | Lines |
|------|-------|
| `inspector/mod.rs` | 148 |
| `inspector/transform.rs` | 162 |
| `inspector/material.rs` | 122 |
| `inspector/camera.rs` | 168 |

## Final Status

| File | Before | After | Status |
|------|--------|-------|--------|
| `ui.rs` | 698 | 486 | ✅ |
| `viewport/mod.rs` | 682 | 271 | ✅ |
| `inspector.rs` | 558 | 148 | ✅ |
| `lifecycle.rs` | 528 | 528 | ⚠️ Close to limit |

## Testing

- `cargo check` passes without warnings
- `cargo fmt` applied
- Build successful

---

## Update 2026-01-01: New Systems Implemented

### Console Panel

| File | Lines |
|------|-------|
| `panels/console/mod.rs` | ~300 |
| `panels/console/log_capture.rs` | ~100 |

Features:
- Thread-safe log capture
- Level filters (E/W/I/D)
- Text search
- Copy to clipboard
- Visible by default

### Advanced Prefab System

| File | Lines |
|------|-------|
| `prefab/mod.rs` | ~100 |
| `prefab/overrides.rs` | ~150 |
| `prefab/instance.rs` | ~100 |
| `prefab/registry.rs` | ~150 |
| `prefab/resolver.rs` | ~200 |
| `prefab/propagation.rs` | ~100 |

Features:
- Property-level overrides (PropertyPath, PropertyOverrides)
- Nested prefabs with circular reference detection
- PrefabRegistry with hot reload support
- Change propagation system

### Timeline/Sequencer

| File | Lines |
|------|-------|
| `timeline/mod.rs` | ~50 |
| `timeline/sequence.rs` | ~250 |
| `timeline/playback.rs` | ~200 |
| `timeline/cutscene.rs` | ~100 |
| `panels/timeline.rs` | ~500 |

Features:
- Track types: bone, object, camera, events
- Keyframe interpolation with easing
- Playback controls
- CutsceneBuilder for cinematic sequences

### CI Fix

- Added `libasound2-dev` to `.github/workflows/ci.yml` for Linux builds

---

## Editor UI Features - COMPLETED (2026-01-01)

All previously missing UI features have been implemented:

### Timeline Panel Integration ✅
- Added `timeline_panel` field to EditorApp in `state.rs`
- Added `draw_timeline_panel()` method in `ui.rs`
- Added View > Timeline menu toggle
- Added Create > Animation > Create Cutscene and New Timeline Sequence
- Full TimelineAction handling (Play, Pause, Stop, Seek, etc.)

### Property Override Indicators ✅
- Created `panels/inspector/prefab.rs` (~180 lines)
- Orange color (`OVERRIDE_COLOR`) for overridden properties
- "Revert" button for individual properties
- "Revert All" and "Apply to Prefab" buttons
- "Select Prefab" and "Open Prefab" actions
- `PrefabAction` enum for action handling

### Nested Prefab UI ✅
- "Replace Selected with Prefab" submenu in Create > Prefab menu
- Prefab instance info in Inspector (source, path, instance ID)
- Visual indicators in inspector for prefab instances

### Cutscene Builder UI ✅
- Cutscene dialog with quick setup options:
  - Camera Pan (creates camera tracks)
  - Dialogue Scene (creates dialogue and camera tracks)
  - Create Empty (basic cutscene)
- Duration and name input
- Integrated with TimelinePanel

### Prefab Registry Hot Reload ✅
- Registry status display (count, modified indicator)
- "Reload All Prefabs" button
- "Reload Modified Only" button (shows only when modified prefabs exist)
- Added methods to `registry.rs`:
  - `len()`, `is_empty()`
  - `get_modified_prefabs()`
  - `reload()`, `reload_all()`

---

## Files Modified/Created

| File | Changes |
|------|---------|
| `app/state.rs` | Added TimelinePanel import and fields |
| `app/ui.rs` | Added draw_timeline_panel(), draw_cutscene_dialog(), PrefabAction handling |
| `app/menu.rs` | Added View > Timeline/Console, Create > Animation, Prefab reload buttons |
| `app/prefab_actions.rs` | Added reload_all_prefabs(), reload_modified_prefabs(), replace_selected_with_prefab() |
| `panels/inspector/mod.rs` | Added prefab module, updated show() signature |
| `panels/inspector/prefab.rs` | NEW - Prefab section UI (~180 lines) |
| `prefab/registry.rs` | Added len(), is_empty(), get_modified_prefabs(), reload() |

---

## Testing

- All 93 unit tests pass
- `cargo check` passes without warnings
- Editor compiles and runs successfully

---

## Update 2026-01-01: Timeline Entity Linking & Keyframes

### Problem
The Timeline panel had the data structures but no UI for:
1. Linking tracks to scene objects (target_entity)
2. Creating keyframes at specific times

### Solution Implemented

**New TimelineAction variants:**
- `AddTrackWithTarget { track_type, target_entity, name }`
- `SetTrackTarget { track_id, entity_id }`
- `UpdateKeyframe { track_id, keyframe_index, time, easing }`
- `CaptureKeyframe { track_id, time, value }`

**New TimelinePanel fields:**
- `show_add_track_dialog: bool`
- `new_track_type: TrackType`
- `new_track_target: Option<u32>`
- `new_track_name: String`
- `keyframe_time_input: String`

**New methods:**
- `draw_add_track_dialog()` - Modal dialog to create track with type and target
- `draw_keyframe_editor()` - Panel showing selected keyframe properties
- `get_keyframe_value_for_track()` - Captures object values for keyframes
- `detect_keyframe_click()` - Click detection on keyframe diamonds

**UI Features:**
1. **Add Track Dialog**: Type dropdown, Target dropdown (scene objects), Name field
2. **Track Context Menu**: "Set Target" submenu to change target entity
3. **Add Keyframe Button**: `[+ Key]` in toolbar, adds keyframe at current time
4. **Double-click**: Adds keyframe at clicked position
5. **Keyframe Selection**: Click on diamond to select
6. **Keyframe Editor**: Shows time, easing, value; allows editing time/easing

**Files Modified:**
| File | Changes |
|------|---------|
| `panels/timeline.rs` | +300 lines (new fields, dialogs, methods) |
| `app/ui.rs` | Updated show() call, added action handlers |

---

## Update 2026-01-01: Timeline Playback → Scene Objects Connection

### Problem
The Timeline system had all components working in isolation:
- ✅ TimelinePlayer updated time correctly
- ✅ player.sample() returned interpolated values
- ✅ SampleResult contained object_transforms with positions/rotations
- ❌ **BUT NOTHING APPLIED THESE VALUES TO SCENE OBJECTS!**

The chain was broken:
```
Play → player.update() → sample() → values discarded → objects static
```

### Solution Implemented

Added code in `draw_timeline_panel()` after `player.update()` that:
1. Checks if timeline is playing
2. Samples the sequence to get interpolated values
3. Applies position/rotation/scale/visibility/color to scene_objects
4. Applies camera position/target if present

**Code Added to `app/ui.rs` (lines 599-640):**
```rust
if self.timeline_panel.player().is_playing() {
    if let Some(sequence) = self.timeline_panel.sequence().cloned() {
        let sample = self.timeline_panel.player_mut().sample(&sequence);

        for (entity_id, transform) in &sample.object_transforms {
            if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *entity_id) {
                if let Some(pos) = transform.position { obj.position = pos; }
                if let Some(rot) = transform.rotation { obj.rotation = rot; }
                if let Some(scale) = transform.scale { obj.scale = scale; }
                if let Some(visible) = transform.visible { obj.visible = visible; }
                if let Some(color) = transform.color { obj.color = color; }
            }
        }
        // Camera values also applied...
    }
}
```

### Result
Now when user:
1. Creates a track with a target entity
2. Adds keyframes with position/rotation values
3. Presses Play

The objects animate in the viewport according to the timeline keyframes!

---

## Update 2026-01-01: Timeline in Play Game Mode

### Problem
Timeline animation only worked in editor mode. When pressing Play Game (green button), the game window had its own copy of scene objects that wasn't being updated by the timeline.

### Architecture Issue
```
Editor Mode:
  self.scene_objects → draw_timeline_panel() applies values ✅

Play Game Mode:
  game_window.scene_objects (COPY!) → scripts update ✅
  timeline_panel → NOBODY APPLIES VALUES! ❌
```

### Solution Implemented

**1. Fixed AddKeyframe handler** (`ui.rs` line 659):
- Double-click now actually adds keyframes (was only logging)
- Captures current object values based on track type and target entity

**2. Added timeline support in Play Game** (`game_events.rs`):
- New method `update_timeline_for_game(delta)` called during `handle_game_redraw()`
- Updates timeline player with delta time
- Samples timeline and applies transforms to `game_window.scene_objects_mut()`

**3. Auto-start timeline after splash screen** (`game_events.rs`):
- Added `timeline_game_started` flag to EditorApp state
- Timeline resets to beginning when game starts
- Timeline auto-starts only AFTER splash screen ends (checks `!gw.in_splash`)

### Files Modified

| File | Changes |
|------|---------|
| `app/ui.rs` | Fixed AddKeyframe handler (was only logging) |
| `app/state.rs` | Added `timeline_game_started` flag |
| `app/game_events.rs` | Added update_timeline_for_game(), auto-start after splash |

### Usage Guide

1. **Create sequence**: Menu Create > Animation > New Timeline Sequence
2. **Add track**: Click `+` in tracks list
3. **Configure track**: Select Type (Position/Rotation/Scale), Target (scene object), Name
4. **Add keyframe at start**:
   - Select track
   - Move playhead to 0s
   - Click `+ Key`
5. **Move object** in Inspector to final position
6. **Add keyframe at end**:
   - Move playhead to desired time (e.g., 2s)
   - Click `+ Key`
7. **Test in editor**: Click `>` in Timeline toolbar
8. **Test in game**: Click green Play button - timeline auto-starts!

---

## Update 2026-01-01: Timeline Persistence (Save/Load)

### Problem
Timeline data was not being saved with the scene, so all animation work was lost on reload.

### Solution Implemented

**1. Added timeline to SceneData** (`scene.rs`):
```rust
pub struct SceneData {
    // ...existing fields...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<TimelineSequence>,
}
```

**2. Save timeline with scene** (`scene_io.rs`):
```rust
// In save_scene():
scene_data.timeline = self.timeline_panel.sequence().cloned();
```

**3. Load timeline with scene** (`scene_io.rs`):
```rust
// In load_scene():
if let Some(timeline) = scene_data.timeline {
    self.timeline_panel.set_sequence(timeline);
} else {
    self.timeline_panel.clear_sequence();
}
```

**4. Clear timeline on new scene** (`scene_io.rs`):
```rust
// In new_scene():
self.timeline_panel.clear_sequence();
```

**5. Added clear_sequence method** (`panels/timeline.rs`)

### Files Modified

| File | Changes |
|------|---------|
| `scene.rs` | Added `timeline` field to SceneData |
| `app/scene_io.rs` | Save/load timeline, clear on new scene |
| `panels/timeline.rs` | Added clear_sequence() method |

---

## Update 2026-01-01: Timeline UI Improvements

### Problems Fixed

1. **Timeline panel not showing on load**: When loading a scene with saved timeline, the panel now automatically opens
2. **No way to delete/edit keyframes**: Added right-click context menu on keyframes

### Solution Implemented

**1. Auto-show timeline on load** (`scene_io.rs`):
```rust
if let Some(timeline) = scene_data.timeline {
    self.timeline_panel.set_sequence(timeline);
    self.show_timeline = true; // NEW: Show panel
}
```

**2. Keyframe context menu** (`panels/timeline.rs`):
- Added `context_menu_keyframe` field to track right-clicked keyframe
- Right-click on keyframe shows popup menu with:
  - 🗑 Delete Keyframe
  - ✏ Edit Time... (opens keyframe editor)

### Files Modified

| File | Changes |
|------|---------|
| `app/scene_io.rs` | Set `show_timeline = true` when loading timeline |
| `panels/timeline.rs` | Added context menu on keyframe right-click |

---

## Update 2026-01-01: 3D Model (Mesh) Support

### Problem
When importing 3D models (.obj files), only cubes appeared in the viewport instead of the actual model geometry.

### Root Cause
The engine correctly loaded OBJ files via `tobj`, but then:
1. Created `SceneObject::cube()` instead of storing mesh reference
2. Discarded all mesh vertex data
3. Always rendered a hardcoded `cube_mesh`

### Solution Implemented

**1. Added `mesh_path` to SceneObject** (`scene_object.rs`):
```rust
pub struct SceneObject {
    // ...existing fields...
    pub mesh_path: Option<String>,
}
```

**2. Added `mesh_path` to SceneObjectData** (`scene.rs`):
- Added for serialization (save/load scenes with meshes)

**3. Added `Mesh::from_raw()` method** (`render/mesh.rs`):
```rust
pub fn from_raw(positions: &[f32], normals: &[f32], texcoords: &[f32], indices: Vec<u32>) -> Self
```

**4. Added mesh cache to Viewport** (`viewport/mod.rs`):
```rust
pub(crate) mesh_cache: HashMap<String, GpuMesh>,

pub fn get_or_load_mesh(&mut self, device: &wgpu::Device, path: &str) -> Option<&GpuMesh>
```

**5. Added mesh_path to ObjectRenderData** (`viewport/mod.rs`):
```rust
pub struct ObjectRenderData {
    pub mesh_path: Option<String>,
    // ...other fields...
}
```

**6. Fixed import to save mesh_path** (`app/menu.rs`):
```rust
// BEFORE (buggy):
let mut obj = SceneObject::cube(self.next_id, center);

// AFTER (correct):
let mut obj = SceneObject::new(self.next_id, &mesh.name);
obj.position = center;
obj.mesh_path = Some(mesh_path_str.clone());
```

**7. Updated rendering to use mesh from cache** (`viewport/render.rs`):
```rust
if let Some(mesh_path) = &obj.mesh_path {
    if let Some(gpu_mesh) = self.mesh_cache.get(mesh_path) {
        gpu_mesh.draw(&mut pass);
    } else {
        self.cube_mesh.draw(&mut pass);  // Fallback
    }
} else {
    self.cube_mesh.draw(&mut pass);  // No mesh = cube
}
```

**8. Added mesh support to GameWindow** (`game_window/mod.rs`, `game_window/render.rs`):
- Added `mesh_cache: HashMap<String, GpuMesh>` to GameWindow
- Preloads meshes on game start
- Uses correct mesh during rendering

### Files Modified

| File | Changes |
|------|---------|
| `scene_object.rs` | Added `mesh_path` field to SceneObject |
| `scene.rs` | Added `mesh_path` field to SceneObjectData |
| `app/scene_io.rs` | Save/load `mesh_path` with scene |
| `render/mesh.rs` | Added `Mesh::from_raw()` method |
| `viewport/mod.rs` | Added `mesh_cache`, `get_or_load_mesh()`, `mesh_path` to ObjectRenderData |
| `viewport/render.rs` | Use mesh from cache instead of cube_mesh |
| `app/lifecycle.rs` | Preload meshes, include mesh_path in render data |
| `app/menu.rs` | Fixed import to save mesh_path |
| `game_window/mod.rs` | Added mesh_cache, preload meshes |
| `game_window/render.rs` | Use mesh from cache in game mode |

### Result
Now when importing 3D models:
1. The actual model geometry is displayed in the viewport
2. Models are saved/loaded correctly with scenes
3. Models render correctly in Play Game mode

---

## Update 2026-01-01: Animation System with Python API

### Problem
1. The "..." button in Animator inspector didn't open file dialog
2. No support for multiple animations per object
3. No Python API to control animations via scripts

### Solution Implemented

**1. Fixed File Dialog** (`panels/inspector/animator.rs`):
- Added `rfd::FileDialog` for skeleton selection
- Filters: GLTF/GLB animation files

**2. Expanded AnimatorComponent** (`components/animator.rs`):
```rust
pub struct AnimationClip {
    pub name: String,
    pub file_path: Option<String>,  // None = from skeleton file
    pub animation_index: usize,     // Index in GLTF file
    pub speed: f32,
    pub looping: bool,
}

pub struct AnimatorComponent {
    pub skeleton_path: Option<String>,
    pub animations: Vec<AnimationClip>,
    pub current_animation: String,
    pub speed: f32,
    pub playing: bool,
    pub current_time: f32,
}
```
- Added methods: `add_animation()`, `remove_animation()`, `animation_names()`, `set_animation()`

**3. Updated Inspector UI** (`panels/inspector/animator.rs`):
- Skeleton file picker with "..." button
- Animation dropdown for current selection
- Speed slider (0-3x)
- Play/Pause/Stop controls
- Animations list with scroll area:
  - Name field per animation
  - File path (optional, for separate animation files)
  - Animation index in file
  - Per-animation speed
  - Loop checkbox
- "+" button to add new animation
- "🗑" button to remove animation

**4. Added Python API** (`scripting/api.rs`):
```python
# Available functions:
xtreme.get_animations(ctx)           # Returns ['Idle', 'Walk', ...]
xtreme.get_current_animation(ctx)    # Returns 'Idle'
xtreme.play_animation(ctx, 'Walk')   # Switch animation
xtreme.stop_animation(ctx)           # Stop playback
xtreme.set_animation_speed(ctx, 1.5) # Set speed
xtreme.is_animation_playing(ctx)     # Returns True/False
```

**5. Script Context Integration** (`scripting/api.rs`, `app/game_events.rs`):
- `AnimatorData` struct for passing animation info to Python
- `AnimationChange` enum for queuing animation changes
- `register_animator()` method in ScriptContext
- Animation data included in `to_py_dict()`
- Animation changes read back and applied to game objects

### Files Modified

| File | Changes |
|------|---------|
| `components/animator.rs` | Added AnimationClip, expanded AnimatorComponent |
| `components/mod.rs` | Export AnimationClip |
| `panels/inspector/animator.rs` | Complete rewrite with file dialog, animation list UI |
| `scripting/api.rs` | Added AnimatorData, AnimationChange, animation Python functions |
| `scripting/mod.rs` | Export AnimatorData, AnimationChange |
| `app/game_events.rs` | Register animator data, apply animation changes |

### Example Python Script

```python
# player_controller.py
import xtreme

def _update(ctx):
    # Check available animations
    anims = xtreme.get_animations(ctx)

    # Switch animation based on input
    if xtreme.is_key_pressed(ctx, 'W'):
        if xtreme.get_current_animation(ctx) != 'Walk':
            xtreme.play_animation(ctx, 'Walk')
    else:
        if xtreme.get_current_animation(ctx) != 'Idle':
            xtreme.play_animation(ctx, 'Idle')

    # Adjust speed while running
    if xtreme.is_key_pressed(ctx, 'Shift'):
        xtreme.set_animation_speed(ctx, 2.0)
    else:
        xtreme.set_animation_speed(ctx, 1.0)
```

### Note
This implementation provides the data structures and control API for animations.
The actual skeletal animation rendering (bone transforms, skinning, GPU vertex blending)
is a separate feature that requires shader support and mesh skinning implementation.
