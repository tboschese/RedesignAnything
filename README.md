# Redesign Anything

**Redesenhe aplicações web existentes com evidência, design system e preservação de jornadas — agent-first, open source, local-first.**

Fork independente do [Craft Anything](https://github.com/storytold/craft-anything) (MIT OR Apache-2.0). Este repositório é o produto **Redesign Anything**; o Craft continua sendo o motor de estudo/reconstrução.

| | |
|---|---|
| **Repo** | https://github.com/tboschese/RedesignAnything |
| **Licença** | MIT OR Apache-2.0 — ver [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE), [`NOTICE`](NOTICE), [`ATTRIBUTION.md`](ATTRIBUTION.md) |
| **Upstream pin** | `827be601e781…` — [`docs/UPSTREAM.md`](docs/UPSTREAM.md) |
| **Status** | PR-01…PR-11 no `main`; V1 pública **ainda não** lançada — [`docs/STATUS.md`](docs/STATUS.md) |
| **Spec** | [`REDESIGN_SPEC/`](REDESIGN_SPEC/README.md) |

> **Não é clone visual.** Pixel likeness não é gate. O foco é capacidade conhecida, UX justificada (incluindo `keep_as_is`), DS real e checks que falham fechados.

---

## O que faz

1. **Study** — reutiliza captura/evidência do Craft; mapeia rotas e oráculos para capacidades com *unknowns* explícitos.
2. **Plan** — knowledge packs locais, Design Critic (sempre com `keep_as_is`), decisões e plano de transformação tipado.
3. **Build** — stack padrão React + TypeScript (Vite); scaffold com mocks simulados; sem builder aninhado no modo `host_agent`.
4. **Check** — registry de design system, rastreabilidade, Ready nativo (`concept` / `prototype` / `engineering`); sem PluginJudge WASM como gate bloqueante.

Controle padrão: **`host_agent` + `lean`**. O agente do Cursor/Claude/Codex dirige o CLI/MCP; não inicia um segundo builder Craft sem handoff explícito.

---

## Para agentes (comece por aqui)

Cole no Cursor / Claude Code / Codex:

```text
Use este repositório: https://github.com/tboschese/RedesignAnything
Leia README.md, AGENTS.md e REDESIGN_SPEC/AGENT_FIRST.md.
Explique os pré-requisitos antes de instalar qualquer coisa.
Não faça install silencioso. Confira docs/BASELINE.md e docs/UPSTREAM.md.
Depois prepare um projeto kind=redesign (CLI/MCP), preserve jornadas conhecidas
e reporte o que não foi verificado. Não altere o sistema original.
```

Checklist do agente:

1. Ler `AGENTS.md`, `REDESIGN_SPEC/AGENTS_REDESIGN.md`, `docs/pitfalls.md`
2. Confirmar Rust (ver Get started) e, no macOS recente, `DEVELOPER_DIR` / `SDKROOT` se o linker falhar
3. Pedir consentimento antes de `rustup`, brew, npm, etc.
4. Usar ferramentas `redesign_*` / CLI; não inventar endpoints nem alegar cobertura 100%

---

## Get started

### Pré-requisitos

- Rust estável recente (upstream pede 1.97+; baseline deste fork usou 1.100)
- Git
- Opcional: Node (Vite/REA), browser para Study web

**Sem instalação silenciosa.** Peça antes de instalar toolchains.

```bash
git clone https://github.com/tboschese/RedesignAnything.git
cd RedesignAnything
# remoto de produto
git remote -v   # origin → tboschese/RedesignAnything
# opcional: acompanhar Craft
git remote add upstream https://github.com/storytold/craft-anything.git

cargo build -p craft-anything-cli
./target/debug/craft-anything-cli doctor --format text
./target/debug/craft-anything-cli commands --format text | head
```

No macOS, se o link falhar com SDK dos Command Line Tools:

```bash
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
```

### Fluxo mínimo (host agent)

Os binários ainda usam o nome de máquina `craft-anything-cli` (herança do Craft). Comandos Redesign:

```bash
# criar projeto redesign sem iniciar builder aninhado
craft-anything-cli redesign-project-init \
  --request "Redesenhar o formulário de despesas" \
  --target "https://example.test/expenses" \
  --projects-dir "$HOME/RedesignProjects"

# listar kinds / default stack
craft-anything-cli redesign-kinds-list

# ingestão do DS fixture + scaffold React/TS
craft-anything-cli redesign-ds-ingest \
  --project /caminho/do/projeto \
  --package-dir REDESIGN_SPEC/fixtures/design-system \
  --policy strict

craft-anything-cli redesign-scaffold-app --project /caminho/do/projeto

# Study → store, critic, plano, ready
craft-anything-cli redesign-study-map --project /caminho/do/projeto \
  --oracle-path REDESIGN_SPEC/fixtures/dense-form/oracle/scenarios.json
craft-anything-cli redesign-knowledge-retrieve --query "tabela densa com filtros" --tags table,filtering
craft-anything-cli redesign-critic-recommend --problem "formulário denso de despesas"
craft-anything-cli redesign-checks-run --project /caminho/do/projeto
```

Rebuild “clássico” do Craft com kind explícito:

```bash
craft-anything-cli rebuild https://example.test "Redesenhar checkout" \
  --kind redesign \
  --projects-dir "$HOME/RedesignProjects"
```

MCP: o mesmo conjunto de tools (`redesign_*`) via `craft-anything-cli mcp` (consulte `AGENTS.md`). Em sessão `host_agent`, `rebuild_start` sem handoff retorna `controller_conflict`.

---

## O que já existe neste fork

| Área | Estado |
|---|---|
| `kind=redesign` + persistência em `craft.json` | Pronto |
| Store versionado `redesign/` | Pronto |
| Study → capabilities / coverage | Pronto |
| Knowledge packs + Critic (`keep_as_is`) | Pronto |
| Design System registry (strict/advisory) | Pronto |
| Decisões / plano / rastreabilidade | Pronto |
| Scaffold React+TS+Vite com mocks | Pronto |
| Ready nativo (`redesign-ready` no kind) | Pronto |
| Trust gateway Application MCP | Pronto (catálogo/gate) |
| REA Deep | Contratos + fallback; **binário live ainda não** |
| Benchmark A/B/C/D | Dry-run honesto; **sem KPIs inventados / sem resultados publicados** |
| Instaladores / SBOM / rebrand completo | Aberto — [`docs/PR12_RELEASE_CHECKLIST.md`](docs/PR12_RELEASE_CHECKLIST.md) |

Docs por PR: `docs/PR01_*.md` … `docs/PR12_*.md`.

Fixtures: `REDESIGN_SPEC/fixtures/` (dense-form, multi-step-journey, design-system).

---

## Princípios

- **Unknowns explícitos** — nunca 100% de cobertura quando o denominador é incompleto
- **`keep_as_is` é válido** — mudança exige justificativa
- **Sem inventar backend** — `backend_change_required` quando falta contrato atômico
- **Telemetria honesta** — `unavailable` ≠ zero tokens
- **Clean room** — tools `redesign_*` allowlisted; Deep/REA exigem consentimento
- **Não escrever na origem** — saída em `app/` + `redesign/`

---

## Layout do repositório

```text
apps/                 craft-anything, craft-anything-cli, craft-anything-web
crates/               motor Craft (L0–L6) + camada AI com módulos redesign_*
REDESIGN_SPEC/        contratos, schemas, fixtures, plano V1
docs/                 STATUS, UPSTREAM, BASELINE, PRxx
AGENTS.md             regras para agentes neste repo
```

---

## Desenvolvimento

```bash
cargo xtask fmt
cargo test -p craft-anything-ai --lib redesign_
cargo clippy -p craft-anything-ai --all-targets -- -D warnings
cargo xtask layers
cargo xtask secrets
```

Branches: `feature/`, `bugfix/`, … + id do roadmap quando houver. Não faça push para `upstream`.

---

## Upstream Craft

Este fork herda o engine, CLI, MCP e UI do Craft. Documentação e branding Craft/ArtCraft ainda aparecem em partes do tree até o rebrand completo.

- Sync e pin: [`docs/UPSTREAM.md`](docs/UPSTREAM.md)
- Baseline de testes: [`docs/BASELINE.md`](docs/BASELINE.md)
- Craft upstream: https://github.com/storytold/craft-anything

**Nunca faça push para o remoto `upstream`.**

---

## Licença e créditos

Código sob **MIT OR Apache-2.0**, igual ao Craft Anything. Preserve avisos em `NOTICE` e [`ATTRIBUTION.md`](ATTRIBUTION.md). Não redistribua design systems ou dados privados de clientes.

---

## O que a V1 pública ainda exige

Antes de chamar isto de release V1 Impactante, falta (entre outros): installers, SBOM, benchmarks medidos publicados, integração live do REA com consentimento, e a matriz completa de [`REDESIGN_SPEC/V1_IMPACTANTE.md`](REDESIGN_SPEC/V1_IMPACTANTE.md). Checkpoints internos ≠ lançamento.
