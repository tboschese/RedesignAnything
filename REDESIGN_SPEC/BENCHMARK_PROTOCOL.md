# Protocolo de benchmark — Redesign Anything v1.2.0

## Hipótese

H1: REA oferece redesign de UX melhor ou mais confiável do que Craft com instruções de UX, **com o mesmo agente/modelo/fixture** e sem perda de funções críticas conhecidas.

## Braços

- **A — Craft comum:** pedido simples de redesign sem packs especializados.
- **B — Craft + prompt de UX:** instruções detalhadas equivalentes ao conhecimento base, sem ferramentas REA próprias.
- **C — Redesign Anything Lean:** Task Model, packs e checks, sem REA/MCP externos.
- **D — REA Deep (opcional):** C + REA, apenas se houver lacunas específicas.

## Controle experimental

Fixar aplicação/commit, fixtures, credenciais sintéticas, DS quando aplicável, agente, modelo, temperatura quando controlável, limite de chamadas, hardware, tempo e instruções. Executar múltiplas repetições quando possível e registrar variância. Julgadores cegos quanto à origem da proposta. Nunca interpretar ausência de contagem de tokens como zero.

## Métricas e gates

- **Bloqueantes:** resultados funcionais críticos conhecidos; nenhum segredo exportado; nenhum check falho marcado como aprovado; nenhum builder aninhado.
- **Qualidade:** sucesso por tarefa, tempo/erros se medidos, revisão de conteúdo, densidade adequada, conformidade DS, acessibilidade automática + manual em amostra.
- **Cobertura:** descoberto/observado/inferido/desconhecido com denominadores definidos; falsa inferência de regra é defeito.
- **Custo:** duração, chamadas, retries, tokens reportados/cache/estimados/indisponíveis.
- **UX:** avaliadores julgam com rubrica e sem saber qual braço produziu cada versão; o Design Critic não substitui avaliação.

## Decisão antes da execução

Registrar limiares numéricos, número de tarefas, tamanho da amostra, critérios de desempate e orçamento em ADR específico **antes** de gerar resultados. Sem isso, resultado é exploratório, não validação causal. Caso B seja equivalente a C, preferir a arquitetura mais simples. Se D não agregar cobertura, deixá-lo fora do fluxo padrão.

## Estado

**Não executado.** Esta é uma metodologia proposta; nenhum score ou ganho foi medido.

## Complemento v1.3.0 — pré-registro obrigatório

Antes da primeira execução, criar um `BENCHMARK_RUN_PLAN` versionado com: conjunto e quantidade de fixtures; pelo menos uma tarefa densa e uma jornada com estados; critérios de inclusão; número de repetições por braço; configuração do agente/modelo; ordem de execução e aleatorização quando possível; orçamento igual; rubrica cega; critérios de desempate; métricas primárias; tamanho mínimo de efeito e limites de regressão funcional **numéricos definidos antes de observar resultados**. Não inventar thresholds nesta documentação.

Amostras pequenas podem orientar decisões de engenharia, mas não autorizam generalização estatística. Separar resultados exploratórios de confirmatórios. Registrar todas as falhas, retries, outliers, versões, prompts, mudanças de ambiente, tokens unavailable e conflitos de interesse de avaliadores. Nenhum LLM pode aprovar sozinho a própria saída.

Oráculos independentes e dados sintéticos devem ser congelados antes de executar A/B/C; consultar `ORACLE_POLICY.md`. A baseline B deve receber instruções UX fortes, sem enfraquecimento artificial. Se C não superar B de maneira útil, preferir a solução mais simples.

## Gate de lançamento V1 Impactante

O benchmark é obrigatório antes da primeira publicação pública; comparar Craft puro, Craft com instruções fortes e Redesign Anything em fixtures distintas, incluindo jornada multietapa e design system real. Pré-registrar métricas e limiares. Se não demonstrar ganho útil contra o baseline com instruções fortes, melhorar a implementação antes do lançamento. Ver `V1_IMPACTANTE.md`.
