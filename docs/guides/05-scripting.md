# Python Scripting

Xtreme Engine supports Python scripting for game logic via pyo3.

## Enabling Scripting

Add the `scripting` feature to your `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = { version = "0.1", features = ["scripting"] }
```

**Requirements:**
- Python 3.9 or higher
- Python in system PATH

## Script Lifecycle

Scripts follow a Godot-like lifecycle:

```python
# my_script.py

def _ready(ctx):
    """Called once when script is first attached"""
    print("Script initialized!")

def _update(ctx, delta):
    """Called every frame"""
    # delta = time since last frame in seconds
    pass

def _physics_update(ctx, delta):
    """Called at fixed physics rate (60 Hz)"""
    pass
```

### Execution Order

1. `_ready(ctx)` - Once when scene loads or script attaches
2. `_update(ctx, delta)` - Every frame (variable rate)
3. `_physics_update(ctx, delta)` - Fixed rate (60 Hz default)

## Context Object

The `ctx` parameter provides access to game state:

```python
def _update(ctx, delta):
    # Object information
    object_id = ctx['object_id']      # Current object's ID
    object_name = ctx['name']         # Object name

    # Transform data
    transform = ctx['transform']
    position = transform['position']   # [x, y, z]
    rotation = transform['rotation']   # [x, y, z] euler
    scale = transform['scale']         # [x, y, z]

    # Input state
    input_data = ctx['input']
    pressed_keys = input_data['keys']           # Currently held
    just_pressed = input_data['keys_just_pressed']  # This frame

    # Time
    time = ctx['time']
    total_time = time['elapsed']       # Seconds since start
    current_delta = time['delta']      # Same as delta param
```

## Transform Manipulation

### Reading Transform

```python
def _update(ctx, delta):
    transform = ctx['transform']

    # Get position
    x = transform['position'][0]
    y = transform['position'][1]
    z = transform['position'][2]

    # Get rotation (radians)
    rx = transform['rotation'][0]
    ry = transform['rotation'][1]
    rz = transform['rotation'][2]
```

### Modifying Transform

Return modified values to apply changes:

```python
def _update(ctx, delta):
    transform = ctx['transform']

    # Move forward
    transform['position'][2] -= 5.0 * delta

    # Rotate
    transform['rotation'][1] += 1.0 * delta

    # Return changes
    return {'transform': transform}
```

## Input Handling

### Keyboard Input

```python
# Helper functions (provided by engine)
def is_key_pressed(ctx, key):
    """Check if key is currently held"""
    input = ctx.get('input', {})
    keys = input.get('keys', [])
    return key in keys

def is_key_just_pressed(ctx, key):
    """Check if key was pressed this frame"""
    input = ctx.get('input', {})
    keys = input.get('keys_just_pressed', [])
    return key in keys
```

### Usage Example

```python
speed = 10.0

def _update(ctx, delta):
    transform = ctx['transform']

    # WASD movement
    if is_key_pressed(ctx, 'W'):
        transform['position'][2] -= speed * delta
    if is_key_pressed(ctx, 'S'):
        transform['position'][2] += speed * delta
    if is_key_pressed(ctx, 'A'):
        transform['position'][0] -= speed * delta
    if is_key_pressed(ctx, 'D'):
        transform['position'][0] += speed * delta

    # Jump on space (just pressed)
    if is_key_just_pressed(ctx, 'Space'):
        transform['position'][1] += 5.0

    return {'transform': transform}
```

### Key Names

| Key | Name |
|-----|------|
| Letters | `'A'` - `'Z'` |
| Numbers | `'0'` - `'9'` |
| Arrow Keys | `'Up'`, `'Down'`, `'Left'`, `'Right'` |
| Special | `'Space'`, `'Enter'`, `'Escape'`, `'Tab'` |
| Function | `'F1'` - `'F12'` |
| Modifiers | `'Shift'`, `'Ctrl'`, `'Alt'` |

## Example Scripts

### Player Controller

```python
# player.py - Basic player movement

speed = 10.0
turn_speed = 2.0

def _ready(ctx):
    print(f"Player {ctx['name']} spawned at {ctx['transform']['position']}")

def _update(ctx, delta):
    transform = ctx['transform']

    # Movement
    move_x = 0.0
    move_z = 0.0

    if is_key_pressed(ctx, 'W'):
        move_z = -1.0
    elif is_key_pressed(ctx, 'S'):
        move_z = 1.0

    if is_key_pressed(ctx, 'A'):
        move_x = -1.0
    elif is_key_pressed(ctx, 'D'):
        move_x = 1.0

    # Apply movement
    transform['position'][0] += move_x * speed * delta
    transform['position'][2] += move_z * speed * delta

    # Rotation with Q/E
    if is_key_pressed(ctx, 'Q'):
        transform['rotation'][1] += turn_speed * delta
    if is_key_pressed(ctx, 'E'):
        transform['rotation'][1] -= turn_speed * delta

    return {'transform': transform}
```

### Rotating Object

```python
# rotate.py - Continuous rotation

rotation_speed = [0.0, 1.0, 0.0]  # Rotate around Y

def _update(ctx, delta):
    transform = ctx['transform']

    transform['rotation'][0] += rotation_speed[0] * delta
    transform['rotation'][1] += rotation_speed[1] * delta
    transform['rotation'][2] += rotation_speed[2] * delta

    return {'transform': transform}
```

### Bouncing Object

```python
# bounce.py - Simple bouncing animation

import math

amplitude = 2.0
frequency = 2.0
base_height = 0.0
elapsed = 0.0

def _ready(ctx):
    global base_height
    base_height = ctx['transform']['position'][1]

def _update(ctx, delta):
    global elapsed
    elapsed += delta

    transform = ctx['transform']

    # Sine wave bounce
    offset = math.sin(elapsed * frequency * math.pi * 2) * amplitude
    transform['position'][1] = base_height + abs(offset)

    return {'transform': transform}
```

### Following Camera

```python
# follow_camera.py - Camera that follows a target

target_offset = [0.0, 5.0, 10.0]  # Behind and above
smoothing = 5.0

def _update(ctx, delta):
    transform = ctx['transform']

    # Get target position (would need to query from engine)
    # For now, use fixed position
    target_pos = [0.0, 0.0, 0.0]

    # Calculate desired position
    desired = [
        target_pos[0] + target_offset[0],
        target_pos[1] + target_offset[1],
        target_pos[2] + target_offset[2],
    ]

    # Smooth movement
    for i in range(3):
        diff = desired[i] - transform['position'][i]
        transform['position'][i] += diff * smoothing * delta

    return {'transform': transform}
```

## Attaching Scripts

### In Editor

1. Select an object in Hierarchy
2. In Inspector, find Scripts section
3. Click **+ Add Script**
4. Select Python file from Asset Browser

### In Code

```rust
use xtreme::scripting::*;

// Initialize runtime
let mut runtime = ScriptRuntime::new();
runtime.set_scripts_dir("./assets/scripts".into());
runtime.initialize()?;

// Load script for object
let script_id = runtime.load_script("player.py", object_id)?;

// Call lifecycle methods
runtime.call_ready(script_id, &context)?;

// In update loop
runtime.call_update(script_id, delta_time, &context)?;
```

## Best Practices

### 1. Use Global State Sparingly

```python
# Good: Use ctx for state when possible
def _update(ctx, delta):
    transform = ctx['transform']
    # ...

# Careful: Global state persists between calls
counter = 0
def _update(ctx, delta):
    global counter
    counter += 1  # Accumulates!
```

### 2. Handle Missing Data

```python
def _update(ctx, delta):
    # Safe access with defaults
    input = ctx.get('input', {})
    keys = input.get('keys', [])

    # Or with try/except
    try:
        position = ctx['transform']['position']
    except KeyError:
        position = [0, 0, 0]
```

### 3. Keep Scripts Focused

```python
# Good: One responsibility per script
# player_movement.py - Just handles movement
# player_combat.py - Just handles combat

# Bad: Everything in one script
# player.py - Movement, combat, inventory, UI, etc.
```

### 4. Use Delta Time

```python
# Good: Framerate independent
position[0] += speed * delta

# Bad: Depends on framerate
position[0] += speed
```

## Debugging

### Print Statements

```python
def _update(ctx, delta):
    print(f"Position: {ctx['transform']['position']}")
    print(f"Delta: {delta}")
```

Output appears in the console/terminal running the editor.

### Error Handling

Script errors are caught and logged:

```
[ERROR] Script error in player.py: KeyError: 'nonexistent'
```

The game continues running, but the script may not behave correctly.

## Limitations

1. **No Direct Object Queries**: Scripts can only modify their own object
2. **No Cross-Script Communication**: Scripts are isolated
3. **Performance**: Python is slower than Rust - use for logic, not heavy computation
4. **Threading**: Scripts run on main thread - avoid blocking operations
