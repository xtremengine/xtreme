# xtreme.py - Mock module for development/testing outside the engine
# Em runtime, o modulo real e injetado pelo Rust via pyo3
# Este arquivo permite testar scripts localmente com Python puro

from typing import Dict, List, Any, Optional

_mock_mode = True

def get_position(ctx: Dict[str, Any], object_id: Optional[int] = None) -> List[float]:
    """Retorna a posicao [x, y, z] do objeto"""
    return ctx.get("position", [0.0, 0.0, 0.0])

def translate(ctx: Dict[str, Any], dx: float, dy: float, dz: float) -> None:
    """Move o objeto por (dx, dy, dz)"""
    pos = ctx.get("position", [0.0, 0.0, 0.0])
    pos[0] += dx
    pos[1] += dy
    pos[2] += dz
    ctx["position"] = pos
    ctx["_position_changed"] = True

def rotate(ctx: Dict[str, Any], rx: float, ry: float, rz: float) -> None:
    """Rotaciona o objeto por (rx, ry, rz) em radianos"""
    rot = ctx.get("rotation", [0.0, 0.0, 0.0])
    rot[0] += rx
    rot[1] += ry
    rot[2] += rz
    ctx["rotation"] = rot
    ctx["_rotation_changed"] = True

def is_key_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Verifica se uma tecla esta pressionada"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys", [])
    return key in keys

def is_key_just_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Verifica se uma tecla foi pressionada neste frame"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys_just_pressed", [])
    return key in keys

def log_info(message: str) -> None:
    """Log de informacao"""
    print(f"[INFO] {message}")

def log_warn(message: str) -> None:
    """Log de aviso"""
    print(f"[WARN] {message}")

def log_error(message: str) -> None:
    """Log de erro"""
    print(f"[ERROR] {message}")


# Teste local
if __name__ == "__main__":
    print("Testando modulo xtreme mock...")
    ctx = {"position": [0.0, 0.5, 0.0], "rotation": [0.0, 0.0, 0.0]}

    log_info("Posicao inicial: " + str(get_position(ctx)))
    translate(ctx, 1.0, 0.0, 0.0)
    log_info("Apos translate(1,0,0): " + str(get_position(ctx)))
    rotate(ctx, 0.0, 0.1, 0.0)
    log_info("Apos rotate(0,0.1,0): " + str(ctx["rotation"]))
    print("Mock OK!")
