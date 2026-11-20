# Redesign Anything — Orquestração, contexto, tokens e limites de execução

> **Status:** contrato proposto para implementação. As faixas abaixo são **hipóteses de planejamento**, não medições de builds do Redesign Anything. Nenhuma tarifa de provedor, assinatura ou gasto monetário é presumida. Leia junto com `SPEC.md`, `CONTRACTS.md`, `AGENT_FIRST.md`, `VALIDATION_MATRIX.md` e o roadmap real do Craft.

## 1. Princípio: um controlador por execução

O Redesign Anything admite duas modalidades mutuamente exclusivas:

1. **`host_agent` (preferencial no modo agent-first):** o usuário está no Cursor/Claude Code/Codex e fornece o repositório ou ferramentas MCP. **Esse agente** toma decisões, escreve o código e controla a execução. O Redesign fornece operações de Study, recuperação de conhecimento, decisões, artefatos de build, checks e relatórios. **Não iniciar `rebuild_start`, outro Claude/Codex/Cursor ou um loop de modelo interno automaticamente.** O agente hospedeiro pode escolher chamar o workflow Craft completo somente após consentimento explícito, encerrando/transferindo a responsabilidade de builder.
2. **`craft_runner`:** usuário inicia o Build/Run pelo desktop ou CLI do Craft. O `RunSet`/runner do Craft possui o builder e seus refinamentos. Os providers de inteligência e investigação **não iniciam outros agentes**; são ferramentas do mesmo fluxo.

**Invariante:** no máximo um controlador de execução de modelo para cada `run_id`. Se o mesmo processo receber solicitações conflitantes, recusar com erro tipado `controller_conflict`. Retomada reutiliza o controlador original ou exige *handoff* explícito, com log de identidade, checkpoint e consentimento. Um MCP conectado não ganha o direito de disparar o runner implicitamente.

**Fallback de modelo:** preferir agente autenticado pelo usuário; backend API ou local somente conforme seleção explícita. O uso de um pacote open source **não** torna inferência gratuita ou compatível com qualquer assinatura. Não copiar, compartilhar ou armazenar credenciais do host.

## 2. Perfis de execução

| Perfil | Padrão / trigger | Conhecimento consultado | Agentes | REA e MCP da aplicação |
|---|---|---|---|---|
| **`lean`** | Default do projeto | Recuperação sob demanda: padrões por tarefa, poucos componentes e evidências resumidas | 1 controlador, crítico no mesmo agente | REA desativado por padrão; MCP de app somente para dúvidas precisas e autorizado |
| **`deep`** | Seleção explícita ou solicitação aprovada diante de uma lacuna concreta | Investigações incrementais, mais estados/jornadas, reconciliação de evidências | 1 controlador; segunda revisão opcional e orçada | REA só em `capture_mode=deep`; app MCP com política de confiança e consentimento |

`deep` **não altera** sozinho o modo de inspeção `clean_room` para `deep`; são eixos distintos. O usuário autoriza separadamente a profundidade da investigação, o uso de ferramentas com efeitos e a saída de dados para modelos remotos. Aplicação MCP e design system podem ser consultados em `lean` se a política permitir e se agregarem contexto necessário.

### Gatilhos para REA (todos requerem Deep autorizado)

- Uma capacidade crítica permanece incerta após o Study normal e pode ser esclarecida por JavaScript acessível/fluxo de API.
- Há conflito específico entre uma evidência observada e uma declaração do MCP.
- Há necessidade de simulação com estado e capturas suficientes para identificar efeitos confirmados.

**Não acionar** por padrão para screenshot, modificação somente de estilo, leitura de guideline, geração de componente ou simples divergência visual. Registrar `probe_reason`, hipóteses, ferramentas chamadas, fatos novos e custo marginal. Uma investigação que não trouxer evidência nova não justifica chamada idêntica novamente.

## 3. Modelo de contexto (carga mínima)

1. **Indexar, não injetar:** knowledge packs locais como catálogo versionado com tags, aplicabilidade e contraindicações. Recuperar primeiro 3–8 regras/candidatos relevantes, com *source ids*; ampliar só após necessidade justificada.
2. **Registry sob demanda:** listar componentes sucintamente, buscar contrato/props/exemplos de apenas componentes escolhidos; nunca colocar o Storybook ou todo o pacote no prompt.
3. **Evidências com referência:** manter capturas completas no Workspace, transmitir somente sumários, ids estáveis e páginas pequenas. Paginação e tamanho máximo de resposta são aplicados antes de devolver resultados ao agente.
4. **Sem prompts monolíticos:** `RedesignKind::brief_sections` informa metas, invariantes, índice de evidências, política DS, critérios e caminho de arquivos; decisões detalhadas são recuperadas por ferramentas quando necessárias.
5. **Critic uma vez por proposta importante:** revisões adicionais somente quando houver falha de gate, decisão nova relevante ou autorização dentro do orçamento. Checks determinísticos têm precedência sobre julgamento verbal.
6. **Cache seguro:** por `target_revision`, `role`, `environment`, `capture_mode`, `design_system_digest`, `knowledge_pack_version`, `tool_version` e `policy_hash`. TTL/expiração para dados mutáveis; não reutilizar evidência de um perfil/tenant em outro. Não guardar segredos em chave ou payload de cache.

## 4. Contabilização de tokens e custo

Definições **distintas** por execução e por etapa:

- `input_tokens_reported`: total de entradas informado pelo backend, **incluindo o que ele classifica como cached quando aplicável**. Não somar os cached novamente ao total.
- `output_tokens_reported`: total de saída do modelo.
- `cached_input_tokens_reported`: subconjunto dos tokens de entrada (ou reportar explicitamente a semântica do provider se diferente).
- `reasoning_tokens_reported`: somente quando o backend informa, registrar separado e documentar se é subconjunto de output, para não duplicar.
- `total_tokens_processed_reported`: soma de categorias mutuamente exclusivas, documentando o critério do backend; **não é uma janela de contexto nem o volume necessariamente cobrado**.
- `billable_tokens` e `monetary_cost`: somente quando o provedor/host fornecer método confiável e sua base de cálculo; sem tarifa configurada/fornecida → `null`, nunca `0` presumido.
- `tool_payload_bytes` / `model_context_bytes`: bytes, **não** “tokens exatos”; se estimados por tokenizer identificado, manter `tokens_estimated` separado de `tokens_reported`.
- `accounting_quality`: `provider_reported`, `host_reported`, `estimated`, `unavailable`. Tratar dados parciais por campo e preservar origem e versão da medição.

No modo `host_agent`, o servidor MCP pode **não receber** metadados das mensagens privadas do Cursor/Claude/Codex. Portanto, **não afirmar medição ou enforce de orçamento total em tokens quando o host não expõe telemetria**. Oferecer contadores efetivamente controláveis: chamadas às ferramentas do Redesign, chamadas ao REA, volume de saída, tempo decorrido do serviço, checks e refinamentos que passam pelo Redesign. A UI/relatório mostra “tokens do host indisponíveis” e orienta consulta ao consumo do próprio agente.

No modo `craft_runner`, usar `usage.json`/eventos do runner quando disponíveis e testar contabilidade de cached tokens por backend. Reportar como valor parcial se o agente/backend omitir dados; não simular precisão.

## 5. Limites: advisory vs enforced

**Obrigatórios por implementação:**

- `token_soft_limit`: **aviso**, não bloqueio universal. Pode virar parada verificável apenas quando o controlador reporta tokens e aceita interrupção confiável. Caso contrário, estado `warning_unenforceable`.
- `max_tool_result_chars`: truncamento/paginação no gateway, preservando evidência completa e referência para detalhes.
- `max_mcp_calls` e `max_rea_invocations`: limites de chamadas feitas **através do adapter**, com contadores duráveis e retorno `budget_exhausted` antes da próxima execução.
- `max_critic_passes`, `max_refine_rounds`: limitados no fluxo que o produto controla; se o host realizar refinamentos por fora, o produto não pode restringi-los.
- `max_run_minutes`: deadline para operações/processos supervisionados pelo Redesign, não bloqueia sessão externa de Cursor/Claude.
- `no_progress_limit`: detectar repetição de mesmas ferramentas/argumentos sem evidência nem alteração útil e encerrar o *loop* controlado pelo produto; apresentar motivo e próximos passos.

Ao esgotar budget: persistir checkpoints/evidências, cancelar sidecars próprios, produzir `status: budget_exhausted` sem converter checks não executados em sucesso e solicitar decisão para ampliar limites. **Nunca repetir automaticamente uma operação cara** quando ela falhou por orçamento ou acesso negado.

## 6. Faixas de tokens para planejar benchmarks (não são compromissos)

| Escopo | Tokens processados acumulados — hipótese inicial |
|---|---:|
| Uma tela simples | 150 mil–400 mil |
| Tela corporativa complexa | 400 mil–1,2 milhão |
| Jornada de 3–5 telas | 1–3 milhões |
| Módulo de 10–20 telas | 3–10 milhões |
| Aplicação extensa | Acima de 10 milhões; não estimar valor máximo antes de amostras |

Incluem contexto repetido, entrada e saída ao longo de chamadas, dependendo das capacidades do modelo. **Não** usar essas faixas como preço, consumo incluso no plano, mínimo garantido ou capacidade de contexto. O projeto deve registrar **quantidade de interações, média/p95 por tipo de ferramenta, tokens reportados/estimados, cached share, custo quando observável, tempo, taxa de sucesso e qualidade final**.

Exemplo de **cenário apenas ilustrativo**, tela complexa com 600 mil tokens processados: Study 150 mil; decisão UX 90 mil; Build 240 mil; Check/refinamento 120 mil. O valor não foi aferido em REA nem no Craft do usuário.

## 7. Defaults de engenharia propostos, ajustáveis e não validados

| Parâmetro | Default inicial | Enforcement |
|---|---:|---|
| `execution.controller` | `host_agent` ao entrar por MCP de um agente; `craft_runner` ao iniciar na UI/CLI Build | Exclusivo por run |
| `execution.profile` | `lean` | Aplicar toolset e gating de providers |
| `execution.allow_nested_builder` | `false` | **Hard block** salvo handoff explicitamente aprovado |
| `token_soft_limit` | 750.000 por execução de tela corporativa | Aviso quando há contador; caso contrário indisponível |
| `max_mcp_calls` | 30 chamadas do bridge da aplicação | Hard no gateway próprio |
| `max_rea_invocations` | 2 por etapa de investigação | Hard no adapter |
| `max_critic_passes` | 1 por alternativa aprovada | Hard nos passos controlados |
| `max_tool_result_chars` | 12.000 caracteres por resultado resumido | Hard antes de enviar ao agente; paginação para detalhe |
| `max_run_minutes` | 90 minutos de operações sob controle do Redesign | Hard nos processos próprios, não no agente hospedeiro |
| `no_progress_limit` | 2 passos de execução controlada sem evidência/alteração útil | Hard onde houver observabilidade suficiente |

Os valores são **heurísticos de partida** e não devem impedir um resultado correto por falta de teste. Medir em benchmark e recalibrar por tamanho de tarefa; sem endereçar `max_refine_rounds` duplicadamente (já existe em `checks`). Se o controlador não suportar telemetria, o resultado deve indicar orçamento parcialmente monitorado.

## 8. Benchmark comparativo e critérios de liberação

Para uma mesma fixture, mesmo agente/modelo/modo, perfil, SO, browser, entradas e consentimentos, executar:

A. Craft isolado como baseline.
B. Redesign Lean (mesmo agente + conhecimento local/DS + checks nativos).
C. Redesign Deep (apenas quando gap justificado: REA e/ou MCP da aplicação).

Registrar resultados **por execução, não só por ferramenta**: qualidade das evidências, precisão/recall de regras conhecidas, cenários funcionais, aderência DS, alterações indevidas, tokens *reportados*, cache, duração, ferramentas invocadas e encerramentos por orçamento. Rodar repetições para avaliar variância; não declarar melhoria baseada em execução única. Comparar também número de modelos/controladores ativos (exigir um por run).

**Aceite inicial:** Lean conclui uma fixture com pelo menos um cenário funcional conhecido e DS aplicado; não usa REA sem gatilho, não lança segundo agente, não acessa modelo externo não autorizado; reporta uso somente quando mensurável. Se gerar resultado útil com overhead alto, **não** bloquear o projeto apenas por valor hipotético: registrar e otimizar, definindo threshold numérico após medição real.

## 9. Limites que não podemos prometer

- Assinatura do Cursor/Claude/Codex cobre qualquer volume ou qualquer uso de MCP/CLI.
- Contar tokens privados de um agente externo que não disponibiliza telemetria.
- Proteger a quota do host com um hard cap aplicado por ferramentas externas.
- Qualidade de redesign em qualquer modelo local pequeno.
- Mesma qualidade de UX, tempo de execução ou custo em projetos e agentes diferentes.
- Zero custo financeiro quando o host ou serviço externo possui política de cobrança própria.

**Referências técnicas:** Craft `docs/ROADMAP.md` (R5.3, R5.4), `docs/STATUS.md`, `docs/extending.md`; REA `README.md` e `docs/reference/mcp-contracts.md`; Layout CLI `README.md`. Consulte `SOURCES.md` para URLs e status de implementação.

## Adendo v1.3.0 — segurança do agente e custo de validação

Limites do gateway MCP/CLI não restringem ferramentas nativas do host. Registrar `host_enforcement=unknown|verified` separadamente de `gateway_enforcement`. Benchmarks devem contabilizar custo de verificações independentes e não reduzir testes críticos para caber em orçamento. Em caso de conflito, interromper e reportar `needs_evidence`, nunca promover `Ready`. Política de dados e consentimento em `TRUST_BOUNDARIES.md`.

## Alinhamento v1.4.0 — V1 Impactante

A primeira entrega pública não é MVP. Todas as capacidades e critérios de lançamento estão em `V1_IMPACTANTE.md`. Os checkpoints técnicos internos não reduzem o escopo do release; políticas de segurança, custo e oráculos permanecem obrigatórias.
