# xtreme.py - Xtreme Engine Python API
# This module is injected by Rust at runtime via pyo3
# This file provides mock implementations for IDE support

from typing import Dict, List, Any, Optional

def get_position(ctx: Dict[str, Any], object_id: Optional[int] = None) -> List[float]:
    """Returns the position [x, y, z] of the object"""
    return ctx.get("position", [0.0, 0.0, 0.0])

def set_position(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the position of the object"""
    ctx["position"] = [x, y, z]
    ctx["_position_changed"] = True

def translate(ctx: Dict[str, Any], dx: float, dy: float, dz: float) -> None:
    """Moves the object by (dx, dy, dz)"""
    pos = ctx.get("position", [0.0, 0.0, 0.0])
    pos[0] += dx
    pos[1] += dy
    pos[2] += dz
    ctx["position"] = pos
    ctx["_position_changed"] = True

def get_rotation(ctx: Dict[str, Any]) -> List[float]:
    """Returns the rotation [x, y, z] in radians"""
    return ctx.get("rotation", [0.0, 0.0, 0.0])

def set_rotation(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the rotation of the object in radians"""
    ctx["rotation"] = [x, y, z]
    ctx["_rotation_changed"] = True

def rotate(ctx: Dict[str, Any], rx: float, ry: float, rz: float) -> None:
    """Rotates the object by (rx, ry, rz) in radians"""
    rot = ctx.get("rotation", [0.0, 0.0, 0.0])
    rot[0] += rx
    rot[1] += ry
    rot[2] += rz
    ctx["rotation"] = rot
    ctx["_rotation_changed"] = True

def get_scale(ctx: Dict[str, Any]) -> List[float]:
    """Returns the scale [x, y, z]"""
    return ctx.get("scale", [1.0, 1.0, 1.0])

def set_scale(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the scale of the object"""
    ctx["scale"] = [x, y, z]
    ctx["_scale_changed"] = True

def get_time(ctx: Dict[str, Any]) -> float:
    """Returns the elapsed time since play started"""
    return ctx.get("time", 0.0)

def is_key_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Returns true if the key is currently pressed"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys", [])
    return key in keys

def is_key_just_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Returns true if the key was just pressed this frame"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys_just_pressed", [])
    return key in keys

def log_info(message: str) -> None:
    """Log an info message"""
    print(f"[INFO] {message}")

def log_warn(message: str) -> None:
    """Log a warning message"""
    print(f"[WARN] {message}")

def log_error(message: str) -> None:
    """Log an error message"""
    print(f"[ERROR] {message}")
