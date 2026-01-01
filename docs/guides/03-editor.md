# Visual Editor Guide

The Xtreme Editor is a visual tool for creating and editing game scenes.

## Launching the Editor

```bash
cargo run --example editor
```

## Interface Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  File  Edit  Object  View  Help                          [Menu] │
├─────────────────────────────────────────────────────────────────┤
│  [Select] [Move] [Rotate] [Scale] | Snap: [X] | Grid: 1.0       │
├──────────────┬──────────────────────────────┬───────────────────┤
│              │                              │                   │
│  Hierarchy   │       3D Viewport            │    Inspector      │
│              │                              │                   │
│  > Scene     │                              │  Transform        │
│    - Cube    │         [Gizmos]             │  Position: x y z  │
│    - Player  │                              │  Rotation: x y z  │
│    - Camera  │                              │  Scale:    x y z  │
│              │                              │                   │
│ ─────────────│                              │  Components       │
│  Asset       │                              │  [+ Add]          │
│  Browser     │                              │                   │
├──────────────┴──────────────────────────────┴───────────────────┤
│  Console                                        [E] [W] [I] [D] │
│  [0.12] E [audio] Failed to load clip...                        │
│  [1.23] I [editor] Scene saved                                  │
├─────────────────────────────────────────────────────────────────┤
│  Timeline: "Walk Cycle"                  |< << [>] >> >|        │
│  [Position]  [◆]────[◆]────[◆]                                │
│  [Rotation]  [◆]──────────[◆]                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

### File Operations

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New Scene |
| `Ctrl+O` | Open Scene |
| `Ctrl+S` | Save Scene |
| `Ctrl+Shift+S` | Save Scene As |

### Edit Operations

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+Y` | Redo (alternate) |
| `Ctrl+X` | Cut |
| `Ctrl+C` | Copy |
| `Ctrl+V` | Paste |
| `Delete` | Delete Selected |
| `Ctrl+D` | Duplicate |
| `Ctrl+A` | Select All |

### Transform Tools

| Shortcut | Tool |
|----------|------|
| `Q` | Select Tool |
| `W` | Move Tool |
| `E` | Rotate Tool |
| `R` | Scale Tool |

### View Controls

| Shortcut | Action |
|----------|--------|
| `F` | Focus on Selected |
| `Home` | Frame All Objects |
| `Ctrl+1` | Front View |
| `Ctrl+3` | Side View |
| `Ctrl+7` | Top View |

### Other

| Shortcut | Action |
|----------|--------|
| `H` | Toggle Visibility |
| `F5` | Toggle Play Mode |

## Viewport Navigation

### Mouse Controls

| Action | Control |
|--------|---------|
| Orbit Camera | Right-click + Drag |
| Pan Camera | Middle-click + Drag |
| Zoom | Scroll Wheel |

### Camera Settings

In the right panel, adjust:
- **Distance**: How far the camera is from target
- **Pitch**: Vertical angle (-89 to 89 degrees)
- **Yaw**: Horizontal angle (0 to 360 degrees)
- **Zoom**: Orthographic zoom level

## Creating Objects

### From Toolbar

Click the buttons in the toolbar:
- **+ Cube**: Creates a cube mesh
- **+ Empty**: Creates an empty object (for grouping)
- **+ Camera**: Creates a camera object

### From Hierarchy

Right-click in the Hierarchy panel for context menu options.

## Object Hierarchy

### Parent-Child Relationships

Objects can be organized in a hierarchy:

1. **Parenting**: Right-click object > Set Parent... > Select parent
2. **Unparenting**: Right-click object > Unparent (move to root)

**Benefits of Hierarchy:**
- Child transforms are relative to parent
- Moving parent moves all children
- Useful for complex objects (e.g., character with weapon)

### Example Hierarchy

```
Scene
├── Player           (root object)
│   ├── Body         (child - relative position)
│   ├── Weapon       (child - inherits rotation)
│   └── Camera       (child - follows player)
├── Environment
│   ├── Ground
│   ├── Tree_1
│   └── Tree_2
└── Lights
    ├── Sun
    └── Ambient
```

## Inspector Panel

### Transform Component

Every object has a transform:
- **Position**: World/local position (X, Y, Z)
- **Rotation**: Euler angles in degrees
- **Scale**: Size multiplier

### Object Settings

- **Name**: Display name in hierarchy
- **Visible**: Toggle rendering
- **Color**: Object tint color

### Camera Component

For camera objects:
- **Projection**: Perspective or Orthographic
- **FOV**: Field of view (perspective only)
- **Near/Far**: Clipping planes
- **Main Camera**: Use this camera in Play Mode

### Scripts

Attach Python scripts:
1. Click **+ Add Script**
2. Select `.py` file from Asset Browser
3. Scripts run in Play Mode

## Working with Scenes

### Creating a New Scene

1. `Ctrl+N` or File > New Scene
2. Optionally save current scene first

### Saving Scenes

1. `Ctrl+S` to save (or Save As for new file)
2. Choose location and filename
3. Default format is `.xtrm` (RON format)

### Loading Scenes

1. `Ctrl+O` or File > Open Scene
2. Navigate to `.xtrm` file
3. Scene loads with all objects

### Scene File Format

Scenes use RON (Rusty Object Notation):

```ron
SceneData(
    version: 2,
    name: "My Scene",
    objects: [
        SceneObjectData(
            name: "Cube",
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            visible: true,
            parent_index: None,
            scripts: [],
            camera: None,
        ),
    ],
)
```

## Prefabs

Prefabs are reusable object templates.

### Creating a Prefab

1. Select object(s) in hierarchy
2. File > Save as Prefab
3. Choose filename

### Using a Prefab

1. File > Load Prefab
2. Select `.prefab` file
3. Prefab objects are added to scene

## Play Mode

Test your scene directly in the editor:

1. Press `F5` or click Play button
2. Scene runs with scripts executing
3. Press `F5` again to stop
4. Changes during play are reverted

### Main Camera

In Play Mode, the viewport switches to the Main Camera:
1. Create a camera object
2. In Inspector, check "Main Camera"
3. Enter Play Mode to see through that camera

## Transform Gizmos

### Move Tool (W)

- Drag colored arrows to move on axis
- Drag colored squares to move on plane
- Red = X, Green = Y, Blue = Z

### Rotate Tool (E)

- Drag colored circles to rotate around axis
- Shows rotation in degrees

### Scale Tool (R)

- Drag colored handles to scale on axis
- Drag center cube to uniform scale

## Snap Settings

Enable snapping for precise placement:

1. Click **Snap** checkbox in toolbar
2. Set **Grid Size** (e.g., 1.0 for 1-unit grid)
3. Transforms snap to grid

## Undo/Redo

The editor tracks all changes:

- `Ctrl+Z`: Undo last action
- `Ctrl+Shift+Z`: Redo undone action
- Unlimited undo history

Tracked actions:
- Object creation/deletion
- Transform changes
- Component modifications
- Hierarchy changes

## Tips & Tricks

### Quick Selection

- Click object in Hierarchy or Viewport
- `Ctrl+Click` to add to selection
- `Ctrl+A` to select all

### Duplicate Objects

- `Ctrl+D` duplicates selected objects
- Duplicates appear at same position

### Focus on Object

- Double-click in Hierarchy
- Or select and press `F`
- Camera centers on object

### Reset View

- Press `Home` to frame all objects
- Use `Ctrl+1/3/7` for standard views

## Audio Components

### Audio Source

Add sound effects or music to any object.

**Creating Audio Source:**
1. Create > Audio > Audio Source
2. Or select object > Inspector > Add Component > Audio Source

**Audio Source Settings:**
- **Clip**: Path to audio file (WAV, MP3, OGG, FLAC)
- **Volume**: 0.0 to 1.0 (default: 0.8)
- **Pitch**: 0.5 to 2.0 (default: 1.0)
- **Looping**: Repeat audio when finished
- **Spatial**: Enable 3D positional audio
  - **Min Distance**: Full volume range
  - **Max Distance**: Audio cutoff range
- **Autoplay**: Start playing when game begins

**Preview Controls:**
- **Play**: Test audio in editor
- **Stop**: Stop preview playback

### Audio Listener

Receives audio from spatial Audio Sources.

**Creating Audio Listener:**
1. Create > Audio > Audio Listener
2. Usually attached to main camera

**Settings:**
- **Active**: Enable/disable listener
- **Master Volume**: Overall volume control

> **Tip**: Only one Audio Listener should be active per scene.

## Particle Effects

Create visual effects like fire, smoke, rain, and explosions.

### Creating Particle Emitters

**From Menu:**
1. Create > Effects > Particle Emitter
2. Choose preset: Fire, Smoke, Sparkles, Rain, Explosion, or Custom

**Add to Existing Object:**
1. Select object
2. Inspector > Add Component > Particle Emitter > [Preset]

### Particle Presets

| Preset | Description | Best For |
|--------|-------------|----------|
| Fire | Rising flame particles, orange/red | Torches, campfires |
| Smoke | Billowing gray particles | Chimneys, explosions |
| Sparkles | Glittering golden particles | Magic effects, pickups |
| Rain | Falling blue drops | Weather effects |
| Explosion | Burst of particles | Impacts, destruction |
| Custom | Manual configuration | Any custom effect |

### Particle Settings

**Emission:**
- **Enabled**: Turn emitter on/off
- **Local Space**: Particles follow emitter movement
- **Max Particles**: Maximum alive at once
- **Spawn Rate**: Particles per second

**Lifetime:**
- **Min/Max**: Random lifetime range in seconds

**Appearance:**
- **Start Color**: Initial particle color (with alpha)
- **End Color**: Final color (fade out with alpha=0)
- **Start Size**: Initial particle size
- **End Size**: Final particle size

**Physics:**
- **Gravity**: XYZ force applied to particles
  - Positive Y = rise (fire)
  - Negative Y = fall (rain)

### Example: Custom Fire Effect

```
Emission:
  Max Particles: 5000
  Spawn Rate: 500/s
  Local Space: ✓

Lifetime:
  Min: 0.5s
  Max: 1.5s

Appearance:
  Start Color: Orange (1.0, 0.5, 0.0, 1.0)
  End Color: Red transparent (1.0, 0.0, 0.0, 0.0)
  Start Size: 0.2
  End Size: 0.05

Physics:
  Gravity: (0.0, 3.0, 0.0)  // Rise upward
```

### Tips for Particles

1. **Performance**: Lower max particles for better FPS
2. **Local Space**: Enable for particles attached to moving objects
3. **World Space**: Disable local space for effects like rain
4. **Fade Out**: Set end color alpha to 0 for smooth disappearance
5. **Preview**: Particles render in editor viewport in real-time

## Animator Component

Basic animation component for future skeletal animation support.

**Creating Animated Object:**
1. Create > Animation > Animated Object

**Settings:**
- **Skeleton Path**: GLTF file with skeleton (coming soon)
- **Current State**: Animation state name
- **Speed**: Playback speed multiplier
- **Playing**: Enable/disable playback

## Console Panel

Debug log panel for viewing engine and game logs.

### Opening the Console

The console panel is visible at the bottom of the editor.

### Console Features

**Level Filters:**
- **E**: Show Error messages (red)
- **W**: Show Warning messages (yellow)
- **I**: Show Info messages (default)
- **D**: Show Debug messages (verbose)

**Search:**
- Type in the filter box to search log messages
- Matches message text, target module, and category

**Actions:**
- **Clear**: Remove all log entries
- **Copy**: Copy all visible logs to clipboard

### Log Categories

| Category | Description |
|----------|-------------|
| Engine | Core engine messages |
| Editor | Editor UI and tools |
| Render | Graphics and shaders |
| Audio | Sound playback |
| Script | Python scripting |
| User | Custom game logs |

### Example Log Output

```
[0.12] E [xtreme::audio] Failed to load clip: not_found.wav
[0.45] W [xtreme::editor] Object has no parent
[1.23] I [xtreme::editor] Scene saved: level1.xtrm
[2.00] D [xtreme::render] Shader compiled successfully
```

## Advanced Prefab System

Prefabs now support nested prefabs and property-level overrides.

### Property Overrides

When you modify a property of a prefab instance:
- The property is marked as **overridden** (shown in bold/yellow in Inspector)
- Other properties continue to update from the original prefab
- Use **Revert to Prefab** to restore the original value

**Override Workflow:**
1. Place a prefab instance in the scene
2. Modify any property (position, scale, color, etc.)
3. The modified property becomes an override
4. Original prefab changes won't affect overridden properties

### Nested Prefabs

Prefabs can contain other prefabs:

```
Enemy_Squad.prefab
├── Leader (reference to Enemy.prefab)
├── Soldier_1 (reference to Enemy.prefab)
├── Soldier_2 (reference to Enemy.prefab)
└── Banner (reference to Flag.prefab)
```

**Creating Nested Prefabs:**
1. Create a prefab with objects
2. Replace any object with a prefab reference
3. Save the parent prefab

**Benefits:**
- Reuse common sub-components
- Update base prefab affects all usages
- Override properties at any nesting level

### Prefab Registry

The editor maintains a cache of loaded prefabs:
- Automatic caching for performance
- Hot reload detection for external changes
- Circular reference detection prevents infinite loops

### Serialization

Prefab instances are saved with the scene:
- Prefab reference path
- Overridden property list
- Nested prefab mappings

## Timeline Panel

Keyframe animation editor for objects, cameras, and cutscenes.

### Opening Timeline

View > Timeline (or the timeline appears at the bottom of the editor)

### Timeline Interface

```
+============================================================================+
| Timeline: "Walk Cycle"                              [+] [-] [Loop]         |
+============================================================================+
| |< << [>] >> >|  00:01.5 / 02:00  [30 FPS v] [Snap: 0.1s] [Zoom: 100%]   |
+----------------------------------------------------------------------------+
| TRACKS                  |  0.0    0.5    1.0    1.5    2.0   |            |
|-------------------------|-----|-----|-----|-----|-----|------|            |
| v [Bone: Spine]         |                                    |            |
|   [x] Position          |  [◆]----[◆]--------[◆]            |            |
|   [x] Rotation          |  [◆]--------[◆]----[◆]            |            |
|-------------------------|-----|-----|-----|-----|-----|------|            |
| v [Events]              |  [!]           [!]                 |            |
+----------------------------------------------------------------------------+
| Keyframe: Position @ 0.5s | Value: (0,1.5,0) | Easing: [EaseInOut v]      |
+============================================================================+
```

### Playback Controls

| Button | Action |
|--------|--------|
| `|<` | Go to start |
| `<<` | Step back one frame |
| `>` / `||` | Play / Pause |
| `>>` | Step forward one frame |
| `>|` | Go to end |

### Track Types

**Object Tracks:**
- **Position**: Animate object position (Vec3)
- **Rotation**: Animate object rotation (Vec3 euler or Quat)
- **Scale**: Animate object scale (Vec3)
- **Color**: Animate object color (RGBA)
- **Visibility**: Toggle visibility (bool)

**Camera Tracks:**
- **Camera Position**: Animate camera location
- **Camera Target**: Animate camera look-at point
- **Camera FOV**: Animate field of view

**Skeletal Tracks:**
- **Bone Translation**: Animate bone position
- **Bone Rotation**: Animate bone rotation (quaternion)
- **Bone Scale**: Animate bone scale

**Event Tracks:**
- **Event**: Trigger named events at specific times
- **Dialogue**: Display subtitles with speaker and duration
- **Audio**: Play audio clips at specific times

### Working with Keyframes

**Adding Keyframes:**
1. Move playhead to desired time
2. Click `+` on track or press `K`
3. Set keyframe value

**Editing Keyframes:**
- Click keyframe diamond to select
- Drag to move in time
- Use Inspector to edit value

**Easing Functions:**
| Easing | Description |
|--------|-------------|
| Linear | Constant speed |
| EaseIn | Slow start |
| EaseOut | Slow end |
| EaseInOut | Slow start and end |
| Step | Instant change at end |
| CubicBezier | Custom curve |

### Creating Cutscenes

Use the Timeline for cinematic sequences:

1. Create a new sequence
2. Add camera position/target tracks
3. Add dialogue events for subtitles
4. Add audio events for music/sfx
5. Add trigger events for gameplay

**Cutscene Example:**
```
0.0s - Camera starts at overview position
0.0s - Event: "disable_player_input"
1.0s - Dialogue: "Hero" - "The castle lies ahead."
3.0s - Camera moves to castle gate
5.0s - Event: "open_gate"
6.0s - Dialogue: "Guard" - "Halt! Who goes there?"
8.0s - Event: "enable_player_input"
```

### Timeline Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Space | Play/Pause |
| K | Add keyframe |
| Delete | Delete selected keyframe |
| Home | Go to start |
| End | Go to end |
| +/- | Zoom in/out |

### Serialization

Sequences are saved with the scene in the `sequences` array:
- Each sequence has name, duration, frame rate
- Tracks with all keyframes
- Track groups for organization
