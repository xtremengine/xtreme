# zigzag.py
# Script que faz o objeto mover em zig-zag no eixo X
# Anexar a um objeto via Inspector > Scripts > Add Script...

# Variaveis de estado
direction = 1  # 1 = direita, -1 = esquerda
speed = 3.0    # Velocidade do movimento
min_x = -5.0   # Limite esquerdo
max_x = 5.0    # Limite direito
start_x = 0.0  # Posicao inicial (sera setada no _ready)

def _ready(ctx):
    """Chamado uma vez quando o script e anexado"""
    global start_x

    # Pegar posicao inicial
    pos = ctx.get("position", [0, 0, 0])
    start_x = pos[0]

    print(f"[ZigZag] Iniciado na posicao X={start_x}")
    print(f"[ZigZag] Limites: {start_x + min_x} <-> {start_x + max_x}")

def _update(ctx, delta):
    """Chamado a cada frame"""
    global direction

    # Pegar posicao atual
    pos = ctx.get("position", [0, 0, 0])
    current_x = pos[0]

    # Calcular nova posicao
    move = speed * delta * direction
    new_x = current_x + move

    # Verificar limites e inverter direcao
    if new_x >= start_x + max_x:
        new_x = start_x + max_x
        direction = -1  # Ir para esquerda
    elif new_x <= start_x + min_x:
        new_x = start_x + min_x
        direction = 1   # Ir para direita

    # Atualizar posicao no contexto
    pos[0] = new_x
    ctx["position"] = pos
    ctx["_position_changed"] = True
