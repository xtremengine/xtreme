# Visual Editor Guide

The Xtreme Editor is a visual tool for creating and editing game scenes.

## Launching the Editor

```bash
cargo run --example editor
```

## Interface Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  File  Edit  Object  View  Help                          [Menu]│
├─────────────────────────────────────────────────────────────────┤
│  [Select] [Move] [Rotate] [Scale] | Snap: [X] | Grid: 1.0      │
├──────────────┬──────────────────────────────┬───────────────────┤
│              │                              │                   │
│  Hierarchy   │       3D Viewport            │    Inspector      │
│              │                              │                   │
│  > Scene     │                              │  Transform        │
│    - Cube    │         [Gizmos]             │  Position: x y z  │
│    - Player  │                              │  Rotation: x y z  │
│    - Camera  │                              │  Scale:    x y z  │
│              │                              │                   │
│              │                              │  Components       │
│              │                              │  [+ Add]          │
├──────────────┴──────────────────────────────┴───────────────────┤
│  Asset Browser                                                  │
│  [assets/] > scenes/ models/ scripts/                          │
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
