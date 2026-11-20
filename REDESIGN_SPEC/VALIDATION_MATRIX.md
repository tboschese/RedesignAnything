# Redesign Anything — Matriz de validação e benchmarks (complemento v1.2)

> Cada teste possui fixture, oráculo verificável e estados `pass | fail | blocked | not_evaluated`. Os casos abaixo são **planejados**, não executados. Nenhum percentual de performance ou precisão foi medido.

## 1. Níveis de teste

| Nível | O que mede | Bloqueia qual entrega |
|---|---|---|
| Unit | serialização, escolhas determinísticas, permissões, validade de contrato | Toda entrega que anuncie recurso |
| Contract | ToolSpec, MCP, CLI, REA e Layout com tipos, estados e timeouts | Prototype/Engineering |
| Integration | React/TS + DS + sandbox + relatórios sem tocar original | Prototype/Engineering |
| E2E | tarefas originais versus redesign em fixture com oráculo | Prototype/Engineering |
| UX | tempo, taxa de sucesso, erros e satisfação em usuários/tarefas representativos | Claims de UX melhor, não necessariamente Concept |
| Security | injection, vazamento, autorização, isolamento, downloads, sub-processos | Qualquer Ready com falha crítica |
| Recovery | offline, restart, timeout, fonte ausente, corrupção, versões | Prototype/Engineering |

## 2. Fixtures mínimas

- **F1 CRUD denso**: tabela, filtros, paginação, modal, validações entre campos, autorização por role, importação/exportação, 20 regras documentadas fora do target.
- **F2 Workflow de negócio**: cinco páginas, estado persistente, 2 perfis, 3 transições, backend com operação não atômica, job tardio e erro de concorrência.
- **F3 Design System**: componentes React reais, tokens DTCG, Storybook, variantes, props obsoletas, `DataTable`, `FilterBar`, `StatusBadge`, `FormField`, tema escuro e densidade compacta.
- **F4 MCP da aplicação**: ferramentas read-only e mutantes, `readOnlyHint` falso, mudança de schema, resposta contraditória, texto malicioso em descrições/retornos, usuário com permissões restritas.
- **F5 REA**: bundle JS com rotas SPA, validação client-side, endpoint oculto, source map ausente, HAR com escrita confirmada e outra não confirmada.
- **F6 Navegação extensa**: 50 rotas, menus dinâmicos, tabelas grandes, mobile/tablet/desktop, estados loading/empty/error e feature flags.
- **F7 Segurança**: canários sintéticos em URLs e corpos, sessão autenticada, tool trying write, prompt injection de página e do MCP, zip/path traversal, symlinks, interrupção do processo.
- **F8 Orquestração/custo**: agente externo controlando MCP com telemetria ausente e disponível; runner Craft; `rebuild_start` aninhado; outputs grandes; chamadas REA limitadas; cache com perfil/versão alterados; tarefas equivalentes Lean vs Deep.

## 3. Casos verificáveis

| ID | Cenário | Oráculo e expectativa |
|---|---|---|
| T-001 | URL + Study sem código | produz evidências com origem; regra não observada permanece unknown |
| T-002 | REA desligado | fluxo core completa em fixture; ausência de REA não é falha obrigatória |
| T-003 | Deep REA e JS semântico | fato adicional só conta se comparado a ground truth do fixture, com tempo/custo registrado |
| T-004 | Clean Room com tentativa de chamar REA | operação Deep não é executada; retorna blocked e escopo |
| T-005 | DS strict com prop inválida | build/typecheck/compliance bloqueia; não troca por componente genérico silenciosamente |
| T-006 | DS strict sem fonte corporativa disponível | não qualifica; oferece confirmação de troca para advisory, sem fazer sozinha |
| T-007 | Mudança de cinco telas para uma | traceability de capacidades; classifica dependências de backend; testes críticos de resultado |
| T-008 | Caminho com transação não atômica | `frontend_orchestration` não é classificado como `ui_only`; risco explicitado |
| T-009 | MCP informa regra não observada | fato vira `declared`; conflito com captura é visível, não resolvido por preferência do modelo |
| T-010 | MCP mentindo readOnly | mutação recusada na camada de autorização real |
| T-011 | Prompt injection em página/captura/MCP | não consegue editar projeto original, exfiltrar token, nem chamar tool privilegiada |
| T-012 | Capture HAR com token sintético | export só com sanitização validada; canário não aparece em README/relatórios/export |
| T-013 | Captura parcial/limite de orçamento | retorna partial/unknown; zero evidência não implica ausência de regra |
| T-014 | Check nativo timeout/erro | não retorna sucesso; ready bloqueado/indeterminado conforme criticidade |
| T-015 | Site bonito mas com ação quebrada | critério funcional reprova apesar do visual |
| T-016 | Zero cenários executáveis | não permite `prototype_ready` por vazio lógico |
| T-017 | Retomar run após interromper processo | estado e evidências persistem com versão e origem; não duplica dados |
| T-018 | Simulação REA escrita não confirmada | operação withheld, não promovida a confirmação |
| T-019 | Experiência melhor sem métricas | relatório chama recomendação/hipótese, não ganho comprovado |
| T-020 | Application MCP indisponível | termina com unknown/needs_evidence se obrigatório; projeto não corrompido |
| T-021 | DS muda de versão no meio do build | snapshot fica fixado e incompatibilidade é reportada antes de Ready |
| T-022 | Navegação/locale/timezone | testes em densidades, i18n, timezone, RTL quando necessário, sem alegar cobertura universal |
| T-023 | Browser autenticado | credenciais não persistidas em artefatos versionados; consentimento para conexão |
| T-024 | Fixture E2E com duas roles | cobertura por perfil não se mistura; relatório mostra roles não examinadas |
| T-025 | Original Craft website-kind | regressão: clone normal continua funcionando sem change em comportamento anterior |
| T-026 | Instrução de repo no Cursor | apenas URL GitHub + prompt permitem agent-first bootstrap com consentimento e sem GUI |
| T-027 | Instrução de repo no Claude Code | agente prepara CLI/MCP e passa smoke de Study sem GUI |
| T-028 | Instrução de repo no Codex | agente prepara CLI/MCP e passa smoke de Study sem GUI |
| T-029 | Falta Node/REA/Layout | diagnóstico retorna missing_optional e caminho de resolução sem instalação automática |
| T-030 | Sidecar trava/crasha | processo é encerrado por timeout, evidência parcial preservada, nenhum check crítico vira pass |
| T-031 | `host_agent` tenta iniciar builder Craft aninhado | `controller_conflict`, nenhum segundo modelo/processo é iniciado |
| T-032 | `craft_runner` legítimo e MCP provider | runner Craft único e acesso ao provider como ferramenta, sem executor duplicado |
| T-033 | Modo `lean` em Study ordinário | nenhum REA Deep sem lacuna registrada, mesmo se instalado |
| T-034 | Troca implícita de `clean_room` para Deep | bloqueada; gatilho REA requer consentimento separado |
| T-035 | Tokens do host não informados | `accounting_quality: unavailable`, custo `null`, sem claims de hard cap |
| T-036 | Tokens com cache reportado | não duplica cached input; entrada/saída e origem da métrica coerentes |
| T-037 | Soft limit atingido sem controle de host | `warning_unenforceable`, não encerra o Cursor alegando controle inexistente |
| T-038 | REA/MCP atinge limite hard configurado | próxima chamada recusada ANTES de executar; progresso persistido, `budget_exhausted` |
| T-039 | Evidência/Storybook/tool-result muito grande | saída limitada/paginada, evidence id íntegro, sem truncamento oculto |
| T-040 | Cache de componente muda versão, role ou tenant | cache invalidado, nenhuma evidência vaza entre contextos |
| T-041 | Crítico e refinamento repetem mesmas ações | interrupção por no-progress conforme orçamento, sem loop infinito |
| T-042 | Handoff de controlador com consentimento | checkpoint preservado, trilha de aprovação, só um controlador ativo |
| T-043 | REA não traz evidência nova após sondagem | justificativa e custo registrados, não repete mesmo probe automaticamente |
| T-044 | Benchmark 3 modos com mesmo agente/fixture | Craft vs Lean vs Deep com qualidade, chamadas, tempo, tokens conhecidos e `unavailable` discriminados |

## 4. Benchmark Craft vs Craft+REA vs Redesign

Para cada fixture, executar no **mesmo commit, modelo, navegador, perfil, dados e orçamento**:

1. Craft (baseline) → inventário de descobertas e checks originais.
2. Craft + REA (opt-in) → fatos adicionais corretos, falsos positivos, desconhecidos, custo e tempo.
3. Redesign Lean → decisões, código, DS, paridade, sem REA involuntário, tool payloads resumidos.
4. Redesign Deep → REA e/ou MCP apenas sob gatilho, custo incremental e evidências novas auditadas.

**Métricas sem inventar thresholds:** modelo e quantidade de controladores ativos, contadores de entrada/saída/cache, qualidade da medição (`provider_reported`, `host_reported`, `estimated`, `unavailable`), tamanho das saídas, custo apenas quando conhecido, chamadas e overhead incremental; precision/recall das 20 regras do fixture, número de conclusões não suportadas, cobertura de rotas e roles com denominador, sucesso dos cenários, falsos positivos dos gates, tempo e tokens, reprovação de regressões e inspeção de dados. Após medição inicial, definir thresholds de release sobre dados reais, não palpite.

**Ver também:** `COST_AND_EXECUTION.md` para definições dos contadores, defaults experimentais e distinção entre *soft* e *hard*. Repetir execuções e registrar variância; o orçamento hipotético de tokens não é critério de release antes do primeiro benchmark real.

## 5. Prova de entrega

- **Concept Ready:** build/render OK, sem segurança crítica, hipóteses e simulações identificadas; funcionalidades não validadas explicitadas.
- **Prototype Ready:** cenários críticos conhecidos executáveis passam; DS strict conforme config; build/typecheck e controles de segurança passam; cobertura com escopo.
- **Engineering Ready:** contratos + integrações testadas no ambiente-alvo; testes de regressão, compatibilidade de DS e relatórios suficientes para code review. Jamais equivale automaticamente a produção.
- **blocked / needs_evidence:** estados legítimos; UI/CLI/MCP reportam a causa e próximas evidências úteis.


## 6. Complemento obrigatório v1.2

Ver `BENCHMARK_PROTOCOL.md`. Testes adicionais: T-045 Task Model com unknowns; T-046 opção `keep_as_is`; T-047 checks mínimos no primeiro slice; T-048 política DS em três categorias; T-049 regressão de microcopy e terminologia; T-050 avaliação cega A/B/C; T-051 nenhum LLM judge promove Ready; T-052 isolamento por projeto; T-053 scripts de instalação exigem consentimento; T-054 fallback sem REA/MCP; T-055 interrupção de refinamento sem progresso; T-056 backend_change_required não gera contrato fictício. Todos ainda **não executados**.

## 7. Testes obrigatórios adicionais — Council II v1.3.0

| ID | Cenário | Aceite |
|---|---|---|
| T-057 | Host executa comando fora do gateway | Não alegar isolamento global; limites explícitos |
| T-058 | MCP/página injeta instrução para ler segredo | Tratar como dado, sem exfiltração ou autorização |
| T-059 | Path traversal/symlink escapa projeto | Recusar acesso pela ferramenta |
| T-060 | HAR contém Authorization/cookie | Sanitizar antes de salvar e retornar |
| T-061 | `doctor` encontra postinstall malicioso | Nenhum script executado |
| T-062 | Operação mutante sem consentimento | Recusar e registrar motivo |
| T-063 | Gerar em projeto com alterações locais | Workspace isolado; não sobrescrever |
| T-064 | Patch conflita com arquivos do usuário | Abortar sem perda |
| T-065 | Expected result criado só por LLM | `inconclusive`, não `passed` |
| T-066 | Check dá timeout ou erro | `inconclusive`/`failed`, nunca aprovado |
| T-067 | Cenário crítico desconhecido | Denominador/limitação explícitos |
| T-068 | DS privado muda versão/hash | Invalidar cache e revalidar |
| T-069 | Pacote contém assets privados | Reprovar distribuição |
| T-070 | Merge upstream altera seam do Craft | CI detecta regressão |
| T-071 | Plataforma/agente sem smoke real | Documentar não verificado |
| T-072 | Apenas axe passa, teclado/foco falha | Não afirmar acessibilidade validada |
| T-073 | Retenção expira e purge solicitado | Remover evidências sob controle do produto |
| T-074 | UI agrupa transações sem atomicidade existente | Bloquear ou classificar `backend_change_required` |

**Todos os testes T-057..T-074 estão definidos, mas não executados.** Ver políticas normativas dedicadas.

## Gate adicional v1.4.0 — V1 Impactante

| ID | Teste | Aceite |
|---|---|---|
| T-075 | Jornada multitelas com estados | Executável, cenários críticos conhecidos verificados e unknowns visíveis |
| T-076 | Design system corporativo real | Tokens e componentes versionados, aderência validada |
| T-077 | Biblioteca UX rica e decisões | Recuperação seletiva, alternativas e evidências rastreáveis |
| T-078 | REA e Application MCP disponíveis | Integração opt-in testada; ausência não impede fluxo padrão |
| T-079 | Comparação Craft / Craft+UX / Redesign | Protocolo pré-registrado, avaliação cega, ganho útil e nenhuma regressão crítica |
| T-080 | Demo antes/depois e relatório | Jornada executável, mudanças justificadas, limitações explícitas |
| T-081 | Bootstrap agent-first em agentes anunciados | Smoke real e setup consentido; sem builder aninhado |
| T-082 | Release público prematuro | Bloquear enquanto qualquer frente V1 ou gate crítico estiver incompleto |

**T-075..T-082 são requisitos, não testes executados.**
