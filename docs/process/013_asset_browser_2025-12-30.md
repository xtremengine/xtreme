# Fase 13: Asset Browser

**Data:** 2025-12-30
**Status:** Concluido

## Objetivo

Implementar painel de navegacao de assets (cenas, prefabs, texturas).

## Arquivos Criados

| Arquivo | Linhas |
|---------|--------|
| `src/editor/panels/asset_browser.rs` | ~260 linhas |

## Arquivos Modificados

| Arquivo | Mudanca |
|---------|---------|
| `src/editor/panels/mod.rs` | +3 linhas |
| `src/editor/app/state.rs` | +4 linhas |
| `src/editor/app/ui.rs` | +25 linhas |

## Estrutura

```rust
pub enum AssetType {
    Directory,
    Scene,
    Prefab,
    Texture,
    Unknown,
}

pub struct AssetEntry {
    pub name: String,
    pub path: PathBuf,
    pub asset_type: AssetType,
}

pub enum AssetAction {
    None,
    OpenScene(PathBuf),
    LoadPrefab(PathBuf),
    OpenDirectory(PathBuf),
}

pub struct AssetBrowser {
    root: PathBuf,
    current_path: PathBuf,
    entries: Vec<AssetEntry>,
    selected: Option<usize>,
    filter: String,
    needs_refresh: bool,
}
```

## Funcionalidades

### Navegacao
- Navegacao por diretorios
- Botao "^" para voltar ao diretorio pai
- Botao "R" para refresh
- Exibicao do caminho atual

### Deteccao de Tipos
- `.ron`, `.json` - Scene
- `.prefab` - Prefab
- `.png`, `.jpg`, `.jpeg`, `.bmp`, `.tga` - Texture
- Diretorios marcados como `[D]`

### Interacao
- Click simples: seleciona
- Duplo click em diretorio: navega
- Duplo click em cena: abre (load_scene)
- Duplo click em prefab: carrega (load_prefab)

### Filtro
- Campo de texto para filtrar por nome
- Case insensitive

### UI
- Painel inferior redimensionavel
- Altura padrao: 180px
- Altura minima: 100px
- ScrollArea para lista de arquivos

## Integracao

```rust
// Em EditorApp
pub(crate) asset_browser: AssetBrowser,

// Em draw_ui()
self.draw_asset_browser_panel(&egui_ctx);

// Handler de acoes
match action {
    AssetAction::OpenScene(path) => self.load_scene(path),
    AssetAction::LoadPrefab(path) => self.load_prefab(path),
    _ => {}
}
```

## Verificacao

```bash
cargo check   # Sem erros
```

## Proximos Passos

- Fase 7: Sistema de Scripts (Python via pyo3)
