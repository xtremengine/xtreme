# 015 - Play/Stop Button para Execução de Scripts

**Data:** 2025-12-31
**Status:** Concluído

## Objetivo

Adicionar botão Play/Stop na toolbar para executar scripts em tempo real, similar ao Unity/Godot.

## Comportamento Implementado

1. **Play**: Inicia execução dos scripts
   - Salva estado atual da cena
   - Chama `_ready()` em todos os scripts
   - Inicia loop de `_update(delta)` a cada frame

2. **Stop**: Para execução
   - Para o loop de updates
   - Restaura estado da cena (reset)

3. **Atalho**: F5 para toggle Play/Stop

## Arquivos Modificados

### `src/editor/app/state.rs`
- Adicionados campos:
  - `is_playing: bool`
  - `saved_scene_state: Vec<SceneObject>`
  - `play_time: f32`
  - `last_frame_instant: Option<std::time::Instant>`

### `src/editor/app/actions.rs`
- Adicionados métodos:
  - `start_play()` - Inicia modo play
  - `stop_play()` - Para modo play e restaura cena
  - `toggle_play()` - Toggle entre play/stop
  - `update_scripts()` - Atualiza scripts a cada frame
  - `sync_script_context()` - Sincroniza contexto com objetos da cena
  - `apply_script_changes()` - Aplica mudanças dos scripts aos objetos

### `src/editor/shortcuts.rs`
- Adicionado `EditorAction::TogglePlay`
- Binding F5 → TogglePlay

### `src/editor/app/lifecycle.rs`
- Handler para `EditorAction::TogglePlay`
- Chamada de `update_scripts()` no loop principal

### `src/editor/app/ui.rs`
- Botão Play (verde) / Stop (vermelho) na toolbar
- Display do tempo de execução durante play

## UI Resultante

```
┌─────────────────────────────────────────────────────────────┐
│ [▶ Play] | [Select] [Move] [Rotate] [Scale] | [Center] ... │
└─────────────────────────────────────────────────────────────┘

Durante play:
┌─────────────────────────────────────────────────────────────┐
│ [⏹ Stop] ▶ 3.2s | [Select] [Move] [Rotate] [Scale] | ...   │
└─────────────────────────────────────────────────────────────┘
```

## Próximos Passos

- Modularizar `actions.rs` (arquivo muito grande)
- Testar com scripts zigzag
