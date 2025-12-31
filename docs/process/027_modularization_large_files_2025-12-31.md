# Modularização de Arquivos Grandes

Data: 2025-12-31

## Objetivo

Modularizar arquivos com mais de 400-500 linhas para manter o código organizado e fácil de manter.

## Arquivos Identificados

| Arquivo | Linhas | Status |
|---------|--------|--------|
| game_window.rs | 740 | Modularizado |
| shortcuts.rs | 652 | Modularizado |
| viewport/mod.rs | 593 | Pendente |
| selection.rs | 544 | Pendente |
| prefab.rs | 509 | Pendente |

## game_window.rs -> game_window/

**Antes:** 740 linhas em um único arquivo

**Depois:** 4 módulos separados

```
src/editor/game_window/
├── mod.rs      (~330 linhas) - GameWindow struct, GameSettings, criação
├── camera.rs   (~44 linhas)  - Cálculo de view_projection
├── splash.rs   (~122 linhas) - Splash screen, load_splash_texture()
└── render.rs   (~268 linhas) - render(), helper methods
```

### Responsabilidades

- **mod.rs**: Struct principal, settings, new(), utility methods (id, resize, update, close)
- **camera.rs**: `get_camera_view_projection()` - encontra câmera principal e calcula matriz
- **splash.rs**: `load_splash_texture()`, `splash_alpha()`, `render_splash()`
- **render.rs**: `render()` principal, `calculate_background_color()`, `apply_splash_fade()`, `update_uniforms()`, `render_clear_pass()`, `render_objects()`, `render_overlays()`, `render_fps_overlay()`

## shortcuts.rs -> shortcuts/

**Antes:** 652 linhas em um único arquivo

**Depois:** 4 módulos separados

```
src/editor/shortcuts/
├── mod.rs      (~167 linhas) - ShortcutManager, bindings, process_input()
├── keys.rs     (~278 linhas) - KeyCode enum, from_egui(), to_egui()
├── types.rs    (~134 linhas) - Modifiers, Shortcut structs
└── actions.rs  (~79 linhas)  - EditorAction enum
```

### Responsabilidades

- **mod.rs**: ShortcutManager struct, default bindings, bind/unbind, process_input()
- **keys.rs**: KeyCode enum com 37 teclas, conversão bidirecional com egui::Key
- **types.rs**: Modifiers (ctrl/shift/alt), Shortcut (key + modifiers)
- **actions.rs**: EditorAction enum (29 ações), name() para display

## Benefícios

1. **Isolamento**: Cada responsabilidade em arquivo separado
2. **Manutenção**: Mais fácil encontrar e modificar código
3. **Testes**: Possibilidade de testar módulos independentemente
4. **Colaboração**: Menos conflitos de merge

## Testes

```
cargo test --lib
# 57 tests passed
```

Testes relevantes mantidos funcionando:
- `test_shortcut_display`
- `test_default_bindings`

## Próximos Passos (Opcional)

Considerar modularização futura:
- viewport/mod.rs (593 linhas)
- selection.rs (544 linhas)
- prefab.rs (509 linhas)
