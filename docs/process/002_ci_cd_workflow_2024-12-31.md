# 002 - CI/CD Workflow Setup

**Data:** 2024-12-31
**Status:** Concluido

## Resumo

Configuracao completa de CI/CD com GitHub Actions para o Xtreme Engine.

## Arquivos Criados

```
.github/
├── workflows/
│   ├── ci.yml              # Pipeline CI principal
│   └── release.yml         # Pipeline de release
├── ISSUE_TEMPLATE/
│   ├── bug_report.yml      # Template para bugs
│   └── feature_request.yml # Template para features
├── pull_request_template.md
└── dependabot.yml          # Atualizacao automatica de deps

rustfmt.toml                # Configuracao de formatacao
clippy.toml                 # Configuracao de linting
deny.toml                   # Auditoria de licencas/seguranca
```

## Workflow CI (`ci.yml`)

### Jobs

1. **check** - Verificacao rapida de compilacao
   - Runs on: ubuntu-latest
   - Comando: `cargo check --all-features`

2. **test** - Suite de testes
   - Matriz: Windows, Linux, macOS
   - Rust: stable + nightly (Linux)
   - Comandos: `cargo test` (debug + release)

3. **fmt** - Verificacao de formatacao
   - Comando: `cargo fmt --all -- --check`

4. **clippy** - Linting
   - Comando: `cargo clippy --all-features -- -D warnings`

5. **docs** - Build de documentacao
   - Comando: `cargo doc --no-deps --all-features`

6. **security** - Auditoria de seguranca
   - Ferramenta: cargo-audit
   - Modo: continue-on-error

7. **coverage** - Cobertura de codigo
   - Ferramenta: cargo-llvm-cov
   - Integracao: Codecov

### Triggers

- Push em `main` e `develop`
- Pull requests para `main` e `develop`

### Dependencias

Jobs com dependencias:
- `test` depende de `check`
- `clippy` depende de `check`
- `docs` depende de `check`
- `coverage` depende de `test`

## Workflow Release (`release.yml`)

### Trigger

- Tags no formato `v*.*.*` (ex: v0.1.0, v1.0.0-beta)

### Jobs

1. **create-release** - Cria release no GitHub
   - Draft automatico
   - Detecta pre-releases (tags com sufixo)

2. **build-release** - Build multi-plataforma
   - Targets:
     - `x86_64-unknown-linux-gnu`
     - `x86_64-pc-windows-msvc`
     - `x86_64-apple-darwin`
     - `aarch64-apple-darwin` (Apple Silicon)

3. **publish-crate** - Publica em crates.io
   - Requer: `CRATES_IO_TOKEN` secret

4. **generate-checksums** - Gera SHA256 dos artefatos

## Configuracoes

### rustfmt.toml

- Edition: 2021
- Max width: 100 caracteres
- Imports agrupados por tipo (std, external, crate)
- Comentarios normalizados

### clippy.toml

- MSRV: 1.75
- Complexidade cognitiva maxima: 30
- Argumentos maximos: 10

### deny.toml

- Licencas permitidas: MIT, Apache-2.0, BSD, etc.
- Vulnerabilidades: deny
- Dependencias nao mantidas: warn

## Dependabot

- Cargo: atualizacoes semanais (segunda-feira)
- GitHub Actions: atualizacoes semanais
- Agrupamento: wgpu, egui, serde

## Templates

### Bug Report
- Campos obrigatorios: descricao, passos, comportamento esperado/atual
- OS, versao Rust, versao Xtreme
- Features habilitadas

### Feature Request
- Problema, solucao proposta, alternativas
- Modulo relacionado
- Disposicao para contribuir

### Pull Request
- Tipo de mudanca (bug fix, feature, breaking, etc.)
- Modulos afetados
- Checklist de qualidade

## Secrets Necessarios

| Secret | Descricao |
|--------|-----------|
| `GITHUB_TOKEN` | Automatico |
| `CRATES_IO_TOKEN` | Publicacao em crates.io |
| `CODECOV_TOKEN` | Upload de cobertura (opcional) |

## Comandos Locais

```bash
# Formatacao
cargo fmt

# Linting
cargo clippy --all-features -- -D warnings

# Testes
cargo test --all-features

# Auditoria de seguranca
cargo install cargo-audit
cargo audit

# Auditoria de licencas
cargo install cargo-deny
cargo deny check

# Cobertura local
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

## Proximos Passos

1. Configurar secrets no repositorio GitHub
2. Adicionar badge de CI no README
3. Configurar branch protection rules
4. Criar CHANGELOG.md para releases
