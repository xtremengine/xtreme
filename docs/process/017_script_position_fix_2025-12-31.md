# 017 - Correção: Scripts Não Moviam Objetos

**Data:** 2025-12-31
**Status:** Concluído

## Problema

Os scripts rodavam (logs apareciam), mas os objetos não se moviam visualmente no viewport.

## Causa Raiz

O fluxo de dados entre Python e Rust estava incompleto:

1. `to_py_dict()` criava um dicionário Python
2. O script modificava `ctx["position"]` via `xtreme.translate()`
3. **BUG**: Nunca lemos as mudanças do PyDict de volta para Rust!
4. `drain_position_changes()` retornava vazio

## Solução

Adicionado método `read_changes_from_dict()` para ler as mudanças do PyDict.

## Arquivos Modificados

### `src/scripting/api.rs` (+50 linhas)

```rust
pub fn read_changes_from_dict(&mut self, dict: &Bound<'_, PyDict>) {
    let object_id = self.current_object;

    // Check if position changed
    if let Ok(Some(changed)) = dict.get_item("_position_changed") {
        if changed.extract::<bool>().unwrap_or(false) {
            if let Ok(Some(pos)) = dict.get_item("position") {
                if let Ok(pos_vec) = pos.extract::<Vec<f32>>() {
                    self.position_changes.push((object_id, [...]));
                }
            }
        }
    }
    // ... rotation e scale também
}
```

### `src/scripting/runtime.rs` (+1 linha)

```rust
// Após call_update:
context.read_changes_from_dict(&ctx_dict);
```

## Fluxo Corrigido

```
1. sync_script_context()
   → Copia posição Rust → ScriptContext

2. call_update()
   → to_py_dict() cria PyDict
   → Script modifica PyDict
   → read_changes_from_dict() ← CORREÇÃO

3. apply_script_changes()
   → drain_position_changes() retorna mudanças
   → Aplica aos objetos
```

## Resultado

- Scripts rodam e movem objetos visualmente
- Posição é restaurada ao clicar Stop
