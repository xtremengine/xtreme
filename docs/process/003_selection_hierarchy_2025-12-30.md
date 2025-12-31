# Fase 3: Selecao e Hierarquia

**Data:** 2025-12-30
**Status:** Concluido

## Objetivo

Implementar sistema de selecao de objetos e painel de hierarquia para o editor visual.

## Arquivos Criados

### src/editor/selection.rs (~180 linhas)
Sistema de selecao com ray picking:
- `ObjectId` - ID unico para objetos
- `SceneObject` - Objeto na cena com transform e material
- `Selection` - Estado de selecao (single/multi)
- `Ray` - Raio para picking 3D
- `pick_object()` - Intersecao ray-AABB

### src/editor/panels/mod.rs (~12 linhas)
Modulo de paineis do editor:
- Exporta HierarchyPanel, ToolbarPanel, InspectorPanel
- Exporta HierarchyAction, ToolbarAction, Tool

### src/editor/panels/hierarchy.rs (~200 linhas)
Painel de hierarquia:
- Lista de objetos com icones
- Busca por nome
- Selecao por click
- Menu de contexto (Delete, Duplicate, Focus)
- Toggle de visibilidade
- Botao para criar objetos

### src/editor/panels/toolbar.rs (~212 linhas)
Barra de ferramentas:
- Tools: Select (Q), Move (W), Rotate (E), Scale (R)
- Transform space: Local/World
- Snap com valores configuraveis
- Quick actions: Center View, Frame All

### src/editor/panels/inspector.rs (~223 linhas)
Inspetor de propriedades:
- Nome e visibilidade
- Transform: Position, Rotation, Scale
- Material: Color picker, Alpha, Presets
- Botoes de reset

## Arquivos Modificados

### src/editor/mod.rs
- Adicionado modulos: selection, panels
- Novos exports: SceneObject, Selection, ObjectId, Ray, pick_object
- Novos exports: HierarchyPanel, ToolbarPanel, InspectorPanel

### src/editor/app.rs
Reescrita completa para usar o novo sistema:
- Troca de `Vec<Vec3>` para `Vec<SceneObject>`
- Integracao de todos os paineis
- Sistema de selecao com ray picking
- Atalhos de teclado (Q/W/E/R, Delete, F)
- Viewport click handler
- Acoes da hierarquia (criar, deletar, duplicar)

## Funcionalidades

### Selecao
- Click no viewport seleciona objeto via ray casting
- Click na hierarquia seleciona objeto
- Multi-select com Ctrl+Click (preparado)
- Highlight de objetos selecionados

### Hierarquia
- Lista todos os objetos da cena
- Busca por nome
- Icone de visibilidade
- Menu de contexto com right-click
- Criar novos objetos

### Toolbar
- Ferramentas com atalhos Q/W/E/R
- Toggle Local/World space
- Snap grid configuravel
- Center view e Frame all

### Inspector
- Editar nome do objeto
- Editar transform (pos, rot, scale)
- Editar cor com color picker
- Presets de cor
- Botoes de reset

## Teste

```bash
cargo run --example editor
```

- Click em objetos no viewport: funciona
- Painel de hierarquia: lista objetos
- Toolbar: botoes funcionais
- Inspector: edita propriedades

## Proximos Passos (Fase 4)

- Inspector mais avancado
- Trait de reflexao para componentes
- Widgets customizados
