# xtreme.pyi - Stub file for IDE autocompletion
# Este modulo e injetado pelo Rust em runtime via pyo3
# O IDE usa este arquivo apenas para autocompletar

from typing import Dict, List, Any, Optional

def get_position(ctx: Dict[str, Any], object_id: Optional[int] = None) -> List[float]:
    """Retorna a posicao [x, y, z] do objeto atual ou especificado"""
    ...

def translate(ctx: Dict[str, Any], dx: float, dy: float, dz: float) -> None:
    """Move o objeto por (dx, dy, dz)"""
    ...

def rotate(ctx: Dict[str, Any], rx: float, ry: float, rz: float) -> None:
    """Rotaciona o objeto por (rx, ry, rz) em radianos"""
    ...

def is_key_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Verifica se uma tecla esta pressionada"""
    ...

def is_key_just_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Verifica se uma tecla foi pressionada neste frame"""
    ...

def log_info(message: str) -> None:
    """Log de informacao"""
    ...

def log_warn(message: str) -> None:
    """Log de aviso"""
    ...

def log_error(message: str) -> None:
    """Log de erro"""
    ...
