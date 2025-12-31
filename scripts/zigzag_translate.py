# zigzag_translate.py
# Usa xtreme.translate() para movimento
# Anexar via: Inspector > Scripts > Add Script... > scripts/zigzag_translate.py

import xtreme

# Estado do movimento
direction = 1      # 1 = direita, -1 = esquerda
speed = 4.0        # Unidades por segundo
distance = 0.0     # Distancia percorrida do centro
max_distance = 5.0 # Limite em cada direcao

def _ready(ctx):
    """Chamado uma vez quando o script e anexado"""
    xtreme.log_info("ZigZag Translate iniciado!")
    xtreme.log_info(f"Velocidade: {speed} u/s, Amplitude: +/-{max_distance}")

def _update(ctx, delta):
    """Chamado a cada frame"""
    global direction, distance

    # Calcular movimento
    move = speed * delta * direction

    # Atualizar distancia acumulada
    distance += move

    # Verificar limites
    if distance >= max_distance:
        # Chegou no limite direito
        overshoot = distance - max_distance
        distance = max_distance - overshoot
        direction = -1
        xtreme.log_info("-> Invertendo para esquerda")

    elif distance <= -max_distance:
        # Chegou no limite esquerdo
        overshoot = -max_distance - distance
        distance = -max_distance + overshoot
        direction = 1
        xtreme.log_info("<- Invertendo para direita")

    # Aplicar movimento no eixo X
    xtreme.translate(ctx, move, 0, 0)

def _physics_update(ctx, delta):
    """Chamado em frequencia fixa (opcional)"""
    pass  # Nao usado neste script
