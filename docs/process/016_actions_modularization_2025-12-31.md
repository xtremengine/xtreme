# 016 - Modularização do actions.rs

**Data:** 2025-12-31
**Status:** Concluído

## Objetivo

O arquivo `src/editor/app/actions.rs` estava com 572 linhas, excedendo o limite recomendado de 300-500 linhas. Foi necessário dividir em módulos menores para melhor manutenibilidade.

## Estrutura Anterior

```
src/editor/app/
├── actions.rs (572 linhas - MUITO GRANDE)
├── state.rs
├── input.rs
├── ui.rs
└── lifecycle.rs
```

## Nova Estrutura

```
src/editor/app/
├── actions.rs (10 linhas - apenas documentação)
├── object_actions.rs (~70 linhas) - create, delete, duplicate, focus
├── history.rs (~130 linhas) - undo, redo, apply_undo_command, apply_redo_command
├── scene_io.rs (~90 linhas) - save_scene, load_scene, new_scene
├── clipboard.rs (~70 linhas) - copy, cut, paste, has_clipboard
├── prefab_actions.rs (~100 linhas) - prefab CRUD operations
├── play_mode.rs (~120 linhas) - start_play, stop_play, update_scripts
├── state.rs
├── input.rs
├── ui.rs
└── lifecycle.rs
```

## Arquivos Criados

| Arquivo | Linhas | Responsabilidade |
|---------|--------|------------------|
| `object_actions.rs` | ~70 | CRUD de objetos |
| `history.rs` | ~130 | Undo/Redo |
| `scene_io.rs` | ~90 | Save/Load de cenas |
| `clipboard.rs` | ~70 | Copy/Cut/Paste |
| `prefab_actions.rs` | ~100 | Operações de prefab |
| `play_mode.rs` | ~120 | Modo Play para scripts |

## Modificações

### `mod.rs`
Adicionados os novos módulos:
```rust
mod object_actions;
mod history;
mod scene_io;
mod clipboard;
mod prefab_actions;
mod play_mode;
```

### `scripting/mod.rs`
Exportado `ObjectTransform` publicamente:
```rust
pub use api::{ScriptContext, ObjectTransform};
```

## Correção Adicional

Corrigido erro de módulo privado:
- `scripting::api` foi tornado público
- `ObjectTransform` agora é exportado corretamente

## Resultado

- Todos os módulos abaixo de 200 linhas
- Melhor separação de responsabilidades
- Mais fácil de navegar e manter
- Compila sem erros (apenas warnings)
