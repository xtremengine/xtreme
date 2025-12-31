# File Explorer + Texturas no Editor

**Data:** 2025-12-31
**Autor:** Claude

## Resumo

Implementacao de duas funcionalidades principais:
1. File Explorer abaixo da Hierarchy com divisoria resizable
2. Suporte a texturas no Inspector COM renderizacao

## Arquivos Modificados

### Estrutura UI (File Explorer)

- `src/editor/app/state.rs` - Adicionado `hierarchy_height: f32` para controlar altura do painel hierarchy
- `src/editor/app/ui.rs` - Reorganizado painel esquerdo com divisoria resizable usando `TopBottomPanel::top` inside `SidePanel::left`

### Asset Browser

- `src/editor/panels/asset_browser.rs`:
  - Adicionado tipo `Script` ao enum `AssetType`
  - Reconhecimento de `.xpfb` (prefab) e `.py` (script)
  - Novos `AssetAction`: `AssignTexture`, `OpenScript`, `ShowInExplorer`, `Delete`
  - Menu de contexto (right-click) com opcoes por tipo de asset

### Texturas - Dados

- `src/editor/selection.rs` - Adicionado campo `texture_path: Option<String>` ao `SceneObject`
- `src/editor/scene.rs`:
  - Campo `texture_path` no `SceneObjectData` com serialization opcional
  - Atualizado teste para incluir o novo campo
- `src/editor/app/scene_io.rs` - Save/load do texture_path

### Texturas - UI

- `src/editor/panels/inspector.rs`:
  - Nova secao "Texture" dentro de Material
  - Mostra nome do arquivo ou "None"
  - Botao "X" para remover textura
  - Botao "Browse Texture..." com file dialog (rfd)

### Texturas - Renderizacao

- `src/shaders/textured_simple.wgsl` - Novo shader simplificado para texturas:
  - Group 0: TransformUniforms (mesmo que basic.wgsl)
  - Group 1: texture_2d + sampler
  - Lighting basico (ambient + diffuse)

- `src/editor/viewport/mod.rs`:
  - Novo struct `ObjectRenderData` com model, color, texture_path
  - Novo struct `CachedTexture` para cache de texturas
  - Campos adicionados ao `Viewport`:
    - `textured_pipeline`
    - `texture_bind_group_layout`
    - `default_texture_bind_group`
    - `texture_cache: HashMap<String, CachedTexture>`
  - Metodo `get_or_load_texture()` para carregar texturas sob demanda
  - Metodo `clear_texture_cache()` para limpar cache

- `src/editor/viewport/render.rs`:
  - Novo metodo `render_objects()` que:
    - Verifica se objeto tem textura no cache
    - Usa `textured_pipeline` com texture bind group para objetos com textura
    - Usa `mesh_pipeline` para objetos sem textura

- `src/editor/app/lifecycle.rs`:
  - `get_render_data()` agora retorna `Vec<ObjectRenderData>` para objetos regulares
  - Pre-carrega texturas antes de chamar render

## Fluxo de Renderizacao

1. `get_render_data()` coleta objetos com `texture_path`
2. Antes de render, `get_or_load_texture()` carrega texturas faltantes no cache
3. `render_objects()`:
   - Para objetos com textura: usa `textured_pipeline` + texture bind group
   - Para objetos sem textura: usa `mesh_pipeline` (basico)

## Como Usar

### Adicionar textura via Inspector
1. Selecione um objeto na cena
2. No Inspector, secao Material, clique "Browse Texture..."
3. Selecione uma imagem (PNG, JPG, JPEG, BMP, TGA)
4. A textura sera aplicada ao objeto

### Adicionar textura via File Explorer
1. Navegue ate a textura no File Explorer (painel inferior esquerdo)
2. Clique com botao direito na textura
3. Selecione "Assign to Selected"
4. A textura sera aplicada ao(s) objeto(s) selecionado(s)

## Testes

- Todos os 57 testes passam
- Clippy passa com apenas 2 warnings sobre campos nao utilizados (aceitavel - sao para uso futuro)
