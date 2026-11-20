# Gap para V1 Impactante (2026-11-20)

Critério vinculante: [`REDESIGN_SPEC/V1_IMPACTANTE.md`](../REDESIGN_SPEC/V1_IMPACTANTE.md).  
Checkpoints PR-01…11 **não** autorizam release público.

## Resumo

| Frente V1 | Situação hoje | Falta para publicação |
|---|---|---|
| Core Craft / redesign lean | Operacional headless (`redesign_*`, kind, store, scaffold, Ready nativo) | Smoke multi-agente anunciado; regressão contínua documentada |
| UX Intelligence | Packs mínimos + Critic + `keep_as_is` | Biblioteca **rica** (mais domínios: jornada, estados, conteúdo empresarial); retrieval já existe |
| Jornadas multi-tela | Study/plan/traceability; scaffold single-page | App gerado com **múltiplas rotas/estados** a partir do plano; demo corporativa completa |
| DS corporativo | Ingest package.json + DTCG + strict check | Caminho Storybook/docs; typecheck real opcional; DS fixture “corporativo” mais completo |
| Preservação funcional | Oráculos fixtures + proxy smoke | **Playwright** (ou runner equivalente) ligado aos oráculos; cenários positivo/negativo executáveis |
| Investigação avançada | MCP trust + REA **contratos**/fallback | **REA live** com Node; Application MCP conectado a server fixture real |
| Quality Engine | Ready nativo, secrets, DS | **axe-core** (ou similar); amostra a11y manual documentada; relatório exportável unificado |
| Agent-first / DX | README + CLI/MCP | Matriz Cursor/Claude/Codex executada e publicada; `doctor` redesign-aware |
| Experiência | CLI/report | Preview/comparação antes-depois no fluxo fixture; UI redesign opcional |
| OSS / distribuição | Licenças herdadas | **SBOM**, installers, rebrand binário, CI do fork, upgrade/rollback |
| Impacto (benchmark) | Dry-run A/B/C/D + fixtures pré-registradas | **Execução medida** + avaliação humana cega; ganho vs braço B; **sem inventar KPI** |

## Prioridade sugerida (código antes de release)

1. Runner de cenários/oráculos + Playwright (fail-closed / `not_evaluated`)
2. axe-core integrado ao Ready (sem certificação WCAG falsa)
3. Scaffold multi-rota a partir do transformation plan
4. Packs UX mais ricos + fixture DS “corporativa”
5. Smoke E2E fixture → `check_results.json`
6. REA live (opt-in)
7. Benchmark publicado + empacotamento (PR-12)

## Explicitamente fora deste momento

- Declarar V1 lançada
- Inventar resultados de benchmark
- Exigir REA/MCP/DS em toda tarefa
