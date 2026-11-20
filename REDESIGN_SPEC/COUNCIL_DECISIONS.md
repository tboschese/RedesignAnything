# Council — decisões consolidadas da revisão v1.2.0

**Status:** simulação de perspectivas profissionais, não reunião com PhDs reais nem consulta a múltiplos modelos independentes. **Escopo:** análise crítica da especificação e da arquitetura; sem benchmark executado. **Decisão:** avançar com P0-minimal e gates mensuráveis.

## Membros simulados e conflitos de interesse

- **Engenharia de software / arquitetura:** manutenção do fork, interfaces, testes e qualidade do código. Veta acoplamento excessivo sem evidência.
- **Engenharia de IA:** qualidade do raciocínio, baselines, tokens e avaliação. Veta “inteligência” não demonstrada.
- **Design / HCI:** tarefas, acessibilidade, conteúdo e métricas de UX. Veta avaliação apenas estética.
- **Product Management:** usuário-alvo, diferenciação, escopo e adoção. Veta P0 inflado.
- **Segurança e privacidade (novo):** isolamento, consentimento, dados e MCP. Veta fluxos que exponham credenciais ou permitam operações não autorizadas.
- **QA / confiabilidade (novo):** oráculos, cenários, não-regressão e falsos positivos. Veta `passed` quando não executado.
- **Developer Experience e OSS (novo):** instalação via URL do repositório, CLI/MCP, documentação, licença, contribuição e upgrades upstream. Veta bootstrap opaco e instalação obrigatória de toda stack.
- **Especialista de domínio / operação empresarial (novo):** regras reais, papéis, tarefas frequentes, integrações e mudanças transacionais. Veta simplificações sem viabilidade comprovada.

## Decisões vinculantes

| ID | Decisão | Racional | Prova de aceite |
|---|---|---|---|
| C-01 | P0-minimal em vertical slice | Evitar subsistemas sem valor validado | Uma jornada Study→Build→Check executada no agente |
| C-02 | Benchmark A/B/C pré-registrado | Isolar diferencial do REA | Mesma fixture/agente/modelo, rubrica cega, relatório |
| C-03 | Task Model explícito | UX depende do trabalho, não da tela | Task Model com unknowns e resultado esperado |
| C-04 | Checks mínimos antecipados | Detectar perda funcional cedo | Build + smoke + traceability desde primeiro slice |
| C-05 | DS em três categorias | Evitar `strict` impraticável | Testes de componentes oficiais/composição/extensão |
| C-06 | Critic consultivo | LLM judge não é prova | Resultado não promove Ready sozinho |
| C-07 | REA e MCP opt-in | Evitar custo e risco sem benefício | Desligados no P0-minimal; degradam graciosamente |
| C-08 | Agente único e limites efetivos | Evitar loops caros e dupla execução | Testes anti-nesting, timeout e usage unavailable |
| C-09 | Proteção por projeto | Evitar vazamento e ações perigosas | Testes de exfiltração, permissões e isolamento |
| C-10 | Critérios de simplificação | Evitar sunk cost | ADR documentando manter/remover subsistemas |
| C-11 | Conteúdo como parte do redesign | Rótulos e erros afetam usabilidade | Revisão de microcopy e terminologia |
| C-12 | Viabilidade de transações | UX não cria serviços inexistentes | Classificação UI-only/orchestration/backend-change |

## Restrições de escopo

O P0-minimal não requer REA, MCP da aplicação, canvas, Figma, ingestão completa de Storybook, segundo agente crítico, inferência paga, multi-tenant ou backend novo. Essas capacidades permanecem opcionais/futuras. A experiência desktop do Craft não deve regredir.

## Registro de hipóteses não comprovadas

H1: Knowledge Packs melhoram a qualidade versus Craft + prompt de UX. H2: adapter de DS produz maior conformidade sem prejudicar tarefas. H3: REA amplia cobertura suficiente para compensar custo. H4: MCP semântico reduz desconhecidos sem introduzir dados incorretos. H5: o fluxo host_agent mantém consumo viável. **Nenhuma hipótese é considerada validada por esta documentação.**
