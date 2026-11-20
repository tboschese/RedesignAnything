# Oracle Policy — resultados esperados independentes v1.3.0

## Problema
Um LLM que infere uma regra, gera código e escreve o teste para a própria hipótese não oferece evidência independente de paridade.

## Tipos de oráculo e precedência
1. `observed_verified`: comportamento observado em aplicação original, com replay e contexto de versão/role/tenant/flags.
2. `contract_verified`: contrato de API, documentação oficial versionada ou regra MCP confirmada por responsável e/ou observação.
3. `human_confirmed`: aceite explícito de especialista do domínio, com identidade e data.
4. `inferred_unverified`: hipótese de agente ou de REA; **não** pode produzir `passed=true` em teste de paridade.
5. `unknown`: ausência de informação; **não** equivale a sucesso.

O código de teste pode ser gerado por IA, mas `expected` e a procedência precisam vir de fontes 1–3. Quando a própria fonte observada tiver cobertura parcial, registrar denominador e estados não observados. Evidências conflitantes bloqueiam gate até reconciliação ou redução explícita do escopo.

## Contrato mínimo
`scenario_id`, `task_id`, `inputs`, `preconditions`, `expected`, `oracle_type`, `evidence_ids`, `environment_fingerprint`, `role`, `tenant`, `feature_flags`, `captured_at`, `oracle_author`, `coverage_scope`, `unknowns`.

## Gate
- `passed`: execução real, assert verificável, oráculo permitido, resultado e log persistidos.
- `failed`: divergência reproduzida.
- `inconclusive`: timeout, indisponibilidade, divergência de ambiente ou oráculo insuficiente.
- `not_run`: sem execução.
- `inferred_unverified` e `unknown` nunca contam como aprovado.
- **Concept Ready** aceita hipóteses identificadas, mas não afirma paridade.
- **Prototype Ready** exige aprovação de todos os cenários críticos *conhecidos e declarados*; cenários desconhecidos aparecem explicitamente no relatório.
- **Engineering Ready** exige oráculos de integração adequados e revisão humana para fluxos críticos; não significa production ready.

## Testes bloqueantes
OP-01 expected produzido apenas por LLM: `inconclusive`.
OP-02 teste falha mas agente declara sucesso: gate falha.
OP-03 timeout/erro no checker: `inconclusive`, não `passed`.
OP-04 evidências de versões/roles distintas: bloqueio ou escopo restrito.
OP-05 cenário crítico omitido do denominador: relatório inválido.
OP-06 simulação stateful não observada tratada como contrato real: bloqueio.
OP-07 oráculo humano sem confirmação rastreável: não promovido.
**Status:** especificado, não implementado.
