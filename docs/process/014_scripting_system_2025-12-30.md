# Fase 14: Sistema de Scripts (Python)

**Data:** 2025-12-30
**Status:** Concluido

## Objetivo

Implementar sistema de scripts Python similar ao Godot para comportamentos de objetos.

## Dependencia

```toml
# Cargo.toml
pyo3 = { version = "0.23", features = ["auto-initialize"], optional = true }

[features]
scripting = ["pyo3"]
```

## Arquivos Criados

| Arquivo | Linhas |
|---------|--------|
| `src/scripting/mod.rs` | ~30 linhas |
| `src/scripting/script.rs` | ~190 linhas |
| `src/scripting/runtime.rs` | ~220 linhas |
| `src/scripting/api.rs` | ~310 linhas |

## Estrutura

### Script (`script.rs`)
```rust
pub type ScriptId = u32;

pub struct Script {
    pub id: ScriptId,
    pub name: String,
    pub path: PathBuf,
    pub source: String,
    pub object_id: u32,
    pub initialized: bool,
}

pub struct ScriptInstance {
    pub id: ScriptId,
    pub object_id: u32,
    module: Py<PyAny>,
    ready_called: bool,
}
```

### Runtime (`runtime.rs`)
```rust
pub struct ScriptRuntime {
    scripts: HashMap<ScriptId, Script>,
    instances: HashMap<ScriptId, ScriptInstance>,
    pending_ready: Vec<ScriptId>,
    next_id: ScriptId,
    initialized: bool,
}

impl ScriptRuntime {
    pub fn attach_script(&mut self, path: PathBuf, object_id: u32) -> Result<ScriptId>;
    pub fn attach_script_source(&mut self, name, source, object_id) -> Result<ScriptId>;
    pub fn detach_script(&mut self, script_id: ScriptId);
    pub fn call_ready(&mut self, context: &mut ScriptContext) -> Result<()>;
    pub fn call_update(&self, context: &mut ScriptContext, delta: f32) -> Result<()>;
    pub fn call_physics_update(&self, context: &mut ScriptContext, delta: f32) -> Result<()>;
    pub fn reload_script(&mut self, script_id: ScriptId) -> Result<()>;
}
```

### Context (`api.rs`)
```rust
pub struct ScriptContext {
    current_object: u32,
    transforms: HashMap<u32, ObjectTransform>,
    visibility: HashMap<u32, bool>,
    names: HashMap<u32, String>,
    input: InputState,
    time: f32,
    // Queues for changes
    position_changes: Vec<(u32, [f32; 3])>,
    rotation_changes: Vec<(u32, [f32; 3])>,
    scale_changes: Vec<(u32, [f32; 3])>,
    spawn_queue: Vec<SpawnRequest>,
    destroy_queue: Vec<u32>,
}
```

## API Python

Scripts Python tem acesso ao modulo `xtreme`:

```python
import xtreme

def _ready(ctx):
    xtreme.log_info("Script ready!")

def _update(ctx, delta):
    if xtreme.is_key_pressed(ctx, "W"):
        xtreme.translate(ctx, 0, 0, -delta * 5.0)
    if xtreme.is_key_pressed(ctx, "S"):
        xtreme.translate(ctx, 0, 0, delta * 5.0)

def _physics_update(ctx, delta):
    xtreme.rotate(ctx, 0, delta, 0)
```

### Funcoes Disponiveis

| Funcao | Descricao |
|--------|-----------|
| `get_position(ctx)` | Obtem posicao do objeto atual |
| `translate(ctx, dx, dy, dz)` | Move o objeto |
| `rotate(ctx, rx, ry, rz)` | Rotaciona o objeto |
| `is_key_pressed(ctx, key)` | Verifica se tecla esta pressionada |
| `is_key_just_pressed(ctx, key)` | Verifica se tecla foi pressionada este frame |
| `log_info(msg)` | Log informativo |
| `log_warn(msg)` | Log de aviso |
| `log_error(msg)` | Log de erro |

## Lifecycle (Similar ao Godot)

1. **_ready(ctx)**: Chamado uma vez quando script e anexado
2. **_update(ctx, delta)**: Chamado a cada frame
3. **_physics_update(ctx, delta)**: Chamado em frequencia fixa (physics tick)

## Uso

```rust
// Compilar com feature
cargo build --features scripting

// No codigo
#[cfg(feature = "scripting")]
{
    use xtreme::scripting::{ScriptRuntime, ScriptContext};

    let mut runtime = ScriptRuntime::new();
    runtime.attach_script("scripts/player.py".into(), object_id)?;

    // No game loop
    runtime.call_update(&mut context, delta_time)?;
}
```

## Verificacao

```bash
cargo check                      # Sem feature (OK)
cargo check --features scripting # Com feature (OK)
```

## Integracao com Editor

### UI no Inspector
- Secao "Scripts" aparece quando objeto esta selecionado
- Lista scripts anexados com nome e ID
- Botao "X" para remover script
- Botao "Add Script..." para anexar novo

### Dialog de Script
- Campo para caminho do script (.py)
- Exemplo: `scripts/player.py`
- Botao "Attach" so ativo se caminho termina em .py

### SceneObject
- Campo `scripts: Vec<u32>` adicionado
- Armazena IDs dos scripts anexados

## Limitacoes Atuais

- Scripts nao podem criar/destruir objetos diretamente (apenas via queues)
- Input state precisa ser atualizado manualmente pelo host
- Nao ha hot-reload automatico (precisa chamar reload_script)

## Proximos Passos (Futuro)

- Hot-reload automatico ao detectar mudancas em arquivos
- Mais funcoes na API (spawn, destroy, get_child, etc.)
- Suporte a Lua como alternativa
