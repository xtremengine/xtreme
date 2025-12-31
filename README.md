# Xtreme Engine

Game engine modular com ECS (Entity Component System), renderizacao isometrica 3D e AI com suporte a Machine Learning.

## Requisitos

- Rust 1.78+ (stable)
- Windows 10/11, Linux ou macOS
- GPU com suporte a Vulkan, DirectX 12, ou Metal

## Instalacao

```bash
# Clone o repositorio
git clone https://github.com/your-username/xtreme-engine.git
cd xtreme-engine

# Compile o projeto
cargo build

# Rode os testes
cargo test --lib
```

## Como Rodar

### Executar Exemplos

```bash
# Exemplo basico de ECS
cargo run --example hello_triangle

# Exemplo de camera isometrica
cargo run --example isometric_camera
```

### Rodar Testes

```bash
# Testes unitarios
cargo test --lib

# Testes com output detalhado
cargo test --lib -- --nocapture

# Verificar compilacao
cargo check
```

### Build de Release

```bash
cargo build --release
```

### Habilitar ML (Machine Learning)

```bash
# Compilar com suporte a ONNX runtime
cargo build --features ml
```

## Estrutura do Projeto

```
xtreme-engine/
├── src/
│   ├── lib.rs           # Entry point da biblioteca
│   ├── core/            # ECS (Entity Component System)
│   │   ├── entity.rs    # Gerenciamento de entidades
│   │   ├── component.rs # Storage de componentes (SparseSet)
│   │   ├── world.rs     # Container principal
│   │   ├── query.rs     # Sistema de queries
│   │   ├── system.rs    # Scheduler de sistemas
│   │   └── archetype.rs # Agrupamento por componentes
│   │
│   ├── render/          # Renderizacao WGPU
│   │   ├── context.rs   # Device/Queue WGPU
│   │   ├── window.rs    # Gerenciamento de janela
│   │   ├── camera.rs    # Camera isometrica/perspectiva
│   │   ├── mesh.rs      # Geometria
│   │   ├── material.rs  # Shaders e materiais
│   │   ├── texture.rs   # Texturas
│   │   └── pipeline.rs  # Render pipeline
│   │
│   ├── input/           # Input handling
│   │   ├── keyboard.rs  # Estado do teclado
│   │   ├── mouse.rs     # Estado do mouse
│   │   └── events.rs    # Fila de eventos
│   │
│   ├── physics/         # Fisica e colisao
│   │   ├── shapes.rs    # AABB, Sphere, OBB
│   │   ├── collision.rs # Deteccao de colisao
│   │   ├── spatial.rs   # Grid espacial
│   │   ├── raycast.rs   # Queries de raycast
│   │   └── movement.rs  # Velocity, RigidBody
│   │
│   ├── ai/              # Inteligencia Artificial
│   │   ├── brain.rs     # Componente AI
│   │   ├── perception.rs# Sensores e FOV
│   │   ├── pathfinding.rs # A* pathfinding
│   │   ├── navmesh.rs   # Navigation mesh
│   │   └── inference.rs # ONNX runtime (feature ml)
│   │
│   ├── math/            # Matematica
│   │   ├── transform.rs # Position, Rotation, Scale
│   │   └── isometric.rs # Coordenadas isometricas
│   │
│   └── utils/           # Utilitarios
│       ├── pool.rs      # Object pooling
│       └── timer.rs     # Delta time
│
├── examples/            # Exemplos executaveis
│   ├── hello_triangle.rs
│   └── isometric_camera.rs
│
└── docs/                # Documentacao
    └── process/         # Log de processos
```

## Modulos

### Core (ECS)

Sistema de Entity Component System inspirado em engines como Bevy e Hecs.

```rust
use xtreme::core::{World, Component};

#[derive(Debug, Clone)]
struct Position { x: f32, y: f32 }
impl Component for Position {}

let mut world = World::new();

// Criar entidade com componentes
let entity = world
    .spawn()
    .with(Position { x: 0.0, y: 0.0 })
    .build();

// Acessar componentes
if let Some(pos) = world.get::<Position>(entity) {
    println!("Position: {:?}", pos);
}

// Query de componentes
for (entity, pos) in world.query_one::<Position>() {
    println!("Entity {:?} at {:?}", entity, pos);
}
```

### Math / Isometrico

Utilitarios para projecao isometrica 2:1.

```rust
use xtreme::math::isometric::{world_to_screen, IsometricConfig};
use glam::Vec3;

let config = IsometricConfig::standard();
let world_pos = Vec3::new(5.0, 0.0, 3.0);
let screen_pos = world_to_screen(world_pos, &config);
```

### Render

Renderizacao baseada em WGPU com suporte a camera isometrica.

```rust
use xtreme::render::{Camera, IsometricCamera};
```

### Physics

Sistema de fisica simples com AABB, raycasting e spatial grid.

```rust
use xtreme::physics::{AABB, Ray, RaycastQuery};
```

### AI

Pathfinding A*, navigation mesh e inferencia de modelos ONNX.

```rust
use xtreme::ai::{AStar, Grid, AIBrain};

// Com feature ml
#[cfg(feature = "ml")]
use xtreme::ai::{OnnxModel, ModelInput};
```

## Features

| Feature | Descricao |
|---------|-----------|
| `default` | Core engine sem ML |
| `ml` | Habilita ONNX runtime para inferencia de modelos |

## Dependencias

- **wgpu** 23.0 - Rendering cross-platform
- **winit** 0.30 - Window management
- **glam** 0.29 - Matematica (vetores, matrizes, quaternions)
- **bytemuck** 1.19 - Conversao de tipos para GPU
- **ort** 2.0 (opcional) - ONNX Runtime para ML

## Status do Projeto

- [x] ECS Core (Entity, Component, World, Query)
- [x] Sistema de Archetypes
- [x] Matematica isometrica
- [x] Transform components
- [x] Sistema de input (keyboard/mouse)
- [x] Shapes de colisao (AABB, Sphere, OBB)
- [x] Pathfinding A*
- [x] Estrutura de AI
- [ ] Render pipeline completo
- [ ] Shaders WGSL
- [ ] Sistema de cenas
- [ ] Audio

## Licenca

MIT License - veja [LICENSE](LICENSE) para detalhes.

## Autor

Felipe Maya
