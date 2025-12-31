# zigzag_simple.py
# Versao simplificada usando a API xtreme
# Anexar via: Inspector > Scripts > Add Script... > scripts/zigzag_simple.py

import math

# Estado global do script
elapsed_time = 0.0

def _ready(ctx):
    """Chamado uma vez quando o script e anexado"""
    try:
        import xtreme
        xtreme.log_info("ZigZag script iniciado!")
    except ImportError:
        print("[ZigZag] Script iniciado!")

def _update(ctx, delta):
    """Chamado a cada frame - movimento senoidal em X"""
    global elapsed_time

    # Acumular tempo
    elapsed_time += delta

    # Movimento senoidal: oscila entre -5 e +5
    # sin() retorna valores entre -1 e 1, multiplicamos por 5
    amplitude = 5.0
    frequency = 1.0  # Uma oscilacao completa por segundo

    # Calcular deslocamento baseado no seno do tempo
    offset_x = math.sin(elapsed_time * frequency * 2 * math.pi) * amplitude

    # Pegar posicao atual
    pos = ctx.get("position", [0.0, 0.5, 0.0])

    # Atualizar apenas X (mantendo Y e Z originais)
    # Usamos a posicao inicial como referencia
    base_x = 0.0  # Posicao base em X
    pos[0] = base_x + offset_x

    # Marcar que posicao mudou
    ctx["position"] = pos
    ctx["_position_changed"] = True
