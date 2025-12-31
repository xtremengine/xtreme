# 25 - Camera Component + Dialog Focus Bug Fix

**Data:** 2025-12-31

## Resumo

Implementadas duas funcionalidades principais:
1. **Bug Fix:** Corrigido bug onde inputs em dialogs afetavam campos na janela principal
2. **Feature:** Implementado componente de camera que pode ser adicionado a entidades

## Problema 1: Bug de Foco em Dialogs

### Causa
Os dialogs (`project_dialog.rs`, `prefab_dialog.rs`) não usavam `ui.push_id()` para isolar seus widgets, causando conflito de IDs com a janela principal do egui.

### Solucao
Adicionado `ui.push_id()` para isolar o contexto dos dialogs:
- `project_dialog.rs`: Adicionado `push_id("project_dialog_content")` e `push_id` para cada aba
- `prefab_dialog.rs`: Adicionado `push_id("prefab_dialog_content")`

## Problema 1.1: Atalhos de Teclado Acionados ao Digitar

### Causa
Os atalhos de teclado eram processados mesmo quando um campo de texto estava em foco. Ao digitar "1", "3" ou "7" em qualquer campo, os atalhos de FrontView/SideView/TopView eram acionados, zerando pitch/yaw da camera do editor.

### Solucao
1. Adicionado verificacao `ctx.wants_keyboard_input()` em `shortcuts.rs:process_input()` para ignorar atalhos quando digitando
2. Mudados atalhos de view de `1/3/7` para `Ctrl+1/3/7`
3. Adicionado `push_id("editor_camera_controls")` para isolamento adicional em `ui.rs`

## Problema 2: Camera como Entidade

### Contexto
A camera do editor era fixa e nao existia forma de adicionar cameras como entidades na hierarquia do jogo.

### Solucao
Criado sistema de componente de camera seguindo padrao ECS:

### Arquivos Criados
- `src/editor/components/mod.rs` - Modulo de componentes
- `src/editor/components/camera.rs` - CameraComponent struct

### Arquivos Modificados
- `src/editor/mod.rs` - Export do modulo components
- `src/editor/selection.rs` - Adicionado campo `camera: Option<CameraComponent>` ao SceneObject
- `src/editor/panels/hierarchy.rs` - Adicionado `HierarchyAction::CreateCamera` e botao "+ Camera"
- `src/editor/panels/inspector.rs` - Adicionado secao "Camera" no inspector
- `src/editor/app/ui.rs` - Processamento do CreateCamera
- `src/editor/app/menu.rs` - "Camera" no menu Create
- `src/editor/scene.rs` - Serializacao do CameraComponent
- `src/editor/app/scene_io.rs` - Save/Load do campo camera
- `src/editor/prefab.rs` - Suporte a camera em prefabs
- `src/editor/app/dialogs/project_dialog.rs` - Bug fix
- `src/editor/app/dialogs/prefab_dialog.rs` - Bug fix

## Estrutura do CameraComponent

```rust
pub struct CameraComponent {
    pub is_main: bool,           // Camera principal
    pub projection: CameraProjection, // Perspective ou Orthographic
    pub priority: i32,           // Prioridade para multiplas cameras
    pub clear_color: [f32; 4],   // Cor de fundo
}

pub enum CameraProjection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { size: f32, near: f32, far: f32 },
}
```

## Uso

1. **Criar Camera:** Menu Create > Camera ou botao "+ Camera" no Hierarchy
2. **Editar Camera:** Selecionar objeto com camera, secao "Camera" aparece no Inspector
3. **Marcar como Main:** Checkbox "Main Camera" define qual camera sera usada no play mode
4. **Hierarquia:** Camera pode ser child de outras entidades (ex: seguir personagem)
5. **Camera do Editor:** Permanece inalterada e independente

## Play Mode

Quando o usuario clica em Play:
- O GameWindow busca a entidade com `camera.is_main = true`
- Se houver multiplas, usa a com maior `priority`
- A view/projection e calculada usando a posicao/rotacao da entidade
- A `clear_color` da camera e usada como cor de fundo
- Se nenhuma camera for encontrada, usa camera fallback padrao

## Arquivos Adicionais Modificados

- `src/editor/game_window.rs` - Usa CameraComponent da entidade em vez de IsometricCamera
- `src/editor/app/lifecycle.rs` - Removido parametro camera do GameWindow::new()

## Visualizacao da Camera no Editor

### Implementacao
Cameras sao renderizadas como **piramides wireframe** (apenas linhas) em vez de cubos solidos, para melhor visualizacao da direcao da camera.

### Geometria da Piramide
- **Apex** (ponta): Na frente, direcao -Z (onde a camera aponta)
- **Base**: 4 cantos formando um quadrado atras
- **8 linhas**: 4 da base + 4 do apex para cada canto

### Arquivos Modificados para Wireframe
- `src/editor/viewport/render.rs`: Adicionado metodo `render_camera_wireframes()`
- `src/editor/app/lifecycle.rs`:
  - `get_render_data()` agora retorna tupla `(regular_objects, camera_objects)`
  - `render()` chama `render_camera_wireframes()` separadamente

### Funcionamento
1. `get_render_data()` separa objetos: regulares (cubos) vs cameras (wireframe)
2. `render()` desenha cubos primeiro
3. `render_camera_wireframes()` desenha piramides usando `gizmo_line_pipeline`
4. Vertices ja em world space, usa mesmo uniforms do gizmo

## Compatibilidade

- Cenas existentes carregam normalmente (`#[serde(default)]`)
- Prefabs existentes continuam funcionando
- Play mode funciona mesmo sem camera na cena (usa fallback)
