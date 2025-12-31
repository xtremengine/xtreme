# 001 - Project Setup

**Data:** 2024-12-29
**Status:** Concluído

## Resumo

Setup inicial da game engine ECS isométrica com suporte a IA via Machine Learning.

## Decisões Arquiteturais

### Linguagem: Rust
- Performance próxima a C++
- Segurança de memória garantida pelo compilador
- Ecossistema moderno (Cargo, crates.io)
- Borrow checker previne bugs comuns

### Renderização: WGPU
- API moderna (Vulkan/DX12/Metal)
- Cross-platform
- Rust-native

### Arquitetura: ECS (Entity Component System)
- Data-oriented design
- Cache-friendly (componentes contíguos em memória)
- Paralelização facilitada
- Composição sobre herança

### IA: Abordagem Híbrida
- **Runtime (Rust):** ONNX Runtime via `ort` crate
- **Treinamento (Python):** PyTorch + Stable-Baselines3
- Modelos exportados em formato ONNX

## Estrutura de Arquivos Criada

```
xtreme/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── entity.rs      (~300 linhas)
│   │   ├── component.rs   (~400 linhas)
│   │   ├── archetype.rs   (~350 linhas)
│   │   ├── world.rs       (~400 linhas)
│   │   ├── query.rs       (~300 linhas)
│   │   └── system.rs      (~350 linhas)
│   ├── render/
│   │   ├── mod.rs
│   │   ├── context.rs
│   │   ├── window.rs
│   │   ├── camera.rs
│   │   ├── vertex.rs
│   │   ├── mesh.rs
│   │   ├── material.rs
│   │   ├── texture.rs
│   │   └── pipeline.rs
│   ├── input/
│   │   ├── mod.rs
│   │   ├── keyboard.rs
│   │   ├── mouse.rs
│   │   └── events.rs
│   ├── physics/
│   │   ├── mod.rs
│   │   ├── shapes.rs
│   │   ├── collision.rs
│   │   ├── spatial.rs
│   │   ├── raycast.rs
│   │   └── movement.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── brain.rs
│   │   ├── perception.rs
│   │   ├── pathfinding.rs
│   │   ├── navmesh.rs
│   │   └── inference.rs
│   ├── math/
│   │   ├── mod.rs
│   │   ├── transform.rs
│   │   └── isometric.rs
│   └── utils/
│       ├── mod.rs
│       ├── pool.rs
│       └── timer.rs
└── docs/
    └── process/
```

## Dependências

```toml
[dependencies]
wgpu = "23.0"
winit = "0.30"
glam = "0.29"
bytemuck = "1.19"
log = "0.4"
env_logger = "0.11"
thiserror = "2.0"
pollster = "0.4"

[features]
ml = ["ort"]  # Habilita inferência ML
```

## Componentes Implementados

### Core ECS
- [x] `Entity` - ID único com geração para detectar referências inválidas
- [x] `SparseSet` - Storage O(1) para componentes
- [x] `Archetype` - Agrupamento de entidades por conjunto de componentes
- [x] `World` - Container central com spawn/despawn
- [x] `Query` - Iteração sobre componentes
- [x] `SystemScheduler` - Execução ordenada de sistemas por estágios

### Render (Stubs)
- [x] `IsometricCamera` - Câmera ortográfica configurável
- [x] `Mesh`, `Material`, `Texture` - Estruturas básicas

### Input
- [x] `Keyboard` - Estados de teclas (pressed, held, released)
- [x] `Mouse` - Posição, delta, botões

### Physics
- [x] `AABB`, `Sphere` - Shapes de colisão
- [x] `SpatialGrid` - Broadphase via grid espacial
- [x] `Ray` - Raycast básico

### AI
- [x] `AIBrain` - Componente para agentes inteligentes
- [x] `Sensor`, `PerceptionMemory` - Sistema de percepção
- [x] `AStar` - Pathfinding A* em grid
- [x] `OnnxModel` (placeholder) - Wrapper para inferência ML

### Math
- [x] `Transform` - Posição, rotação, escala
- [x] `IsometricCoord` - Conversão de coordenadas isométricas

### Utils
- [x] `Pool` - Object pool com handles generacionais
- [x] `DeltaTime`, `FixedTimestep` - Controle de tempo

## Próximos Passos

1. Implementar renderização WGPU completa
2. Criar exemplo hello_triangle
3. Implementar câmera isométrica funcional
4. Testar compilação e resolver erros

## Notas

- Cada arquivo mantém limite de 300-500 linhas
- Modularização permite desenvolvimento incremental
- ML é feature opcional (`--features ml`)
