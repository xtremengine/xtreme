# Process: Editor Visual com Egui

**Data:** 2025-12-30
**Fase:** 1 - Infraestrutura do Editor

## Objetivo

Implementar a infraestrutura base do editor visual do Xtreme Engine usando egui + wgpu.

## Arquivos Criados/Modificados

### Novos Arquivos

1. `src/render/egui_integration.rs` (~145 linhas)
   - Wrapper de integracao egui-wgpu-winit
   - Gerencia contexto egui, estado winit e renderer wgpu
   - Metodos: `new()`, `handle_event()`, `begin_frame()`, `end_frame()`, `render()`

2. `src/editor/app.rs` (~305 linhas)
   - EditorApp principal com UI completa
   - Menu bar: File, Edit, View, Help
   - Hierarchy panel: lista de entidades
   - Inspector panel: propriedades da entidade selecionada
   - Status bar: frame count, entity count, resolucao

3. `src/shaders/basic.wgsl` (~80 linhas)
   - Shader WGSL basico para renderizacao 3D
   - Uniforms: view_proj, model, color
   - Iluminacao direcional simples

### Arquivos Modificados

1. `Cargo.toml`
   - Atualizado wgpu 22.1 -> 23
   - Atualizado egui 0.29 -> 0.30
   - Atualizado egui-wgpu 0.29 -> 0.30
   - Atualizado egui-winit 0.29 -> 0.30

2. `src/render/mod.rs`
   - Adicionado modulo `egui_integration`
   - Exportado `EguiIntegration`

3. `src/render/window.rs`
   - Adicionado metodo `raw_event()` ao trait `App`
   - Permite apps receberem eventos winit raw para egui

4. `src/render/pipeline.rs`
   - Ajustado `entry_point` para `Some("vs_main")` (API wgpu 23)

5. `examples/editor.rs`
   - Atualizado para usar `EditorApp` da biblioteca

## Problemas Resolvidos

### 1. Incompatibilidade wgpu/egui-wgpu

**Problema:** egui-wgpu 0.29 usa wgpu 22.1, egui-wgpu 0.30 usa wgpu 23

**Solucao:** Atualizar todas as dependencias para versoes compativeis

### 2. Lifetime `'static` no RenderPass

**Problema:** `egui_wgpu::Renderer::render()` exige `RenderPass<'static>`

**Solucao:** Usar `render_pass.forget_lifetime()` para converter o lifetime

```rust
let render_pass = encoder.begin_render_pass(&descriptor);
let mut render_pass = render_pass.forget_lifetime();
renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
```

### 3. API egui 0.30

**Problema:** `begin_frame()` e `end_frame()` deprecated

**Solucao:** Usar `begin_pass()` e `end_pass()`

## Resultado

Editor visual funcional com:
- Janela 1280x720 redimensionavel
- Menu bar com opcoes File/Edit/View/Help
- Painel Hierarchy para criar/selecionar entidades
- Painel Inspector para ver propriedades
- Status bar com informacoes de debug
- Fechamento com ESC ou File > Exit

## Proximos Passos (Fase 2)

- [ ] Viewport 3D com renderizacao de grid
- [ ] Renderizacao de cubos/meshes no viewport
- [ ] Camera isometrica controlavel

## Referencias

- [egui-wgpu 0.30 docs](https://docs.rs/egui-wgpu/0.30.0)
- [wgpu RenderPass::forget_lifetime](https://docs.rs/wgpu/23.0.0)
