# Redesign Anything — Agent-first, MCP e CLI

> **Estado:** operações `redesign_*` estão no CLI/MCP deste fork (ver README da raiz). Nomes de binário ainda são `craft-anything-cli`. A seção “CLI — interface-alvo” abaixo descreve um wrapper futuro `redesign-anything-cli`; use os comandos `redesign-*` já listados no README até o rebrand.

## Objetivo de compatibilidade

**Padrão de execução:** `host_agent` + `lean`. O agente que recebeu o link mantém o controle e chama ferramentas CLI/MCP do Redesign. Ele **não inicia** outro builder Craft/Claude/Codex sem solicitação/handoff explícito. Consulte `COST_AND_EXECUTION.md` para mecanismos de budget e telemetria.

O usuário deve conseguir iniciar no Cursor, Claude Code, Codex ou outro agente de código informando **apenas a URL do repositório Redesign Anything** e a intenção de instalar/usar. O agente vai ler as instruções, fazer checkout, conferir pré-requisitos, solicitar consentimento para modificações e usar CLI/MCP sem abrir o desktop. Não pressupor que link do GitHub = instalação instantânea, nem que qualquer agente tenha a mesma ferramenta de browser/Git ou MCP.

## Fluxo A — O usuário passa a URL do repo

Prompt previsto para os usuários, após a publicação do fork:

```text
Use este repositório Redesign Anything: https://github.com/tboschese/RedesignAnything
Leia README.md, AGENTS.md e REDESIGN_SPEC/AGENT_FIRST.md. Explique os pré-requisitos antes
de instalar qualquer coisa. Confira a versão do repositório e rode o diagnóstico.
Configure o CLI/MCP quando permitido, sem substituir minha configuração existente.
Depois redesenhe https://app.exemplo.com no modo redesign, preservando jornadas
conhecidas e reportando explicitamente funcionalidades não verificadas.
Não altere os sistemas originais nem envie dados de aplicações privadas a provedores
externos sem minha autorização. Mostre onde ficaram os artefatos e testes.
```

## Fluxo B — Agente já está no projeto

```text
Neste checkout do Redesign Anything, faça um redesign funcional da URL que vou
informar. Use o fluxo Study → Build → Check → Ready, priorize o design system
corporativo local em modo strict e não invente contratos de backend.
```

## Fluxo C — Conexão MCP já configurada

O agente lista operações MCP do Redesign, apresenta escopo e permissões, inicia `redesign_project_*`, consulta `redesign_study_*`, lê decisões/DS, constrói e executa `redesign_check_*`. Os nomes reais são definidos no PR-01/PR-03 conforme contratos no SPEC.md. MCP remoto externo deve ter consentimento e identidade fixada; não assumir que `readOnlyHint` garante segurança.

## CLI — interface-alvo (APENAS PROPOSTA)

```sh
redesign-anything-cli doctor --json
redesign-anything-cli redesign init --url https://app.exemplo.com --mode redesign --config redesign.json
redesign-anything-cli redesign run --project /caminho/do/projeto --json
redesign-anything-cli redesign status --project /caminho/do/projeto --json
redesign-anything-cli redesign report --project /caminho/do/projeto --json
```

**Essas flags ainda não foram implementadas nem verificadas no binário.** Antes de documentar como reais, validar no CLI que o fork efetivamente produzir, mantendo interoperabilidade com comandos upstream.

## Requisitos técnicos obrigatórios

- **Headless parity:** Study, decisões, Build, checks, report, pausas e retomadas disponíveis sem egui.
- **Controlador único:** `host_agent` e `craft_runner` são modos exclusivos. `rebuild_start` interno não é invocado em modo `host_agent`; evitar gastar tokens com um segundo agente executando as mesmas etapas.
- **Lean by default:** Design Intelligence local por recuperação seletiva; REA/critic adicional somente se houver evidência ou aprovação que justifique.
- **Telemetria honesta:** não alegar tokens do Cursor/Claude/Codex quando o host não expuser os contadores; usar limites de ferramentas/tempo controlados pelo produto como fallback.
- **Do not fork the workflow:** aproveitar `RunSet`, `Workspace`, `ToolSpec`, MCP, approvals e `TargetKind`/`Check` do Craft.
- **Autodiagnóstico:** checar Rust, CLI/build, navegador opcional, Node/REA/Layout opcionais, portas e permissões, listando comandos para corrigir, sem instalação silenciosa.
- **Progressive enhancement:** sem REA ou Layout, funcionalidade base permanece; se DS strict foi exigido e está inválido, bloquear a qualificação.
- **Não escrever no projeto original:** saída em `app/` controlado pelo Craft, com diff explícito para edições em repositório existente.
- **Credenciais:** só secret store/ambiente de execução autorizado; jamais gravar token em config, logs, prompts ou artefatos.
- **Proveniência:** cada decisão e teste cita a fonte; dados de MCP da aplicação são `declared`, não `observed` por padrão.
- **Idempotência:** executar novamente o mesmo diagnóstico não cria novos arquivos; reinício retoma projeto versionado.
- **Compatibilidade:** Cursor, Claude Code, Codex testados por uma matriz de capacidades (shell, Git, MCP, approvals); nenhum deles é exigido em exclusividade.
- **Rede:** baixar dependências, clonar fontes, abrir a aplicação alvo e usar modelos remotos devem ser operações explícitas com egress policy.

## Critérios de aceite do agent-first P0

1. Em um agente com shell/Git autorizado, apenas a URL do repo + prompt leva ao checkout e leitura das instruções sem perguntas desnecessárias, fora consentimentos essenciais.
2. Diagnóstico retorna JSON tipado sem abrir janela.
3. A ausência de REA, Layout, Storybook ou MCP da aplicação aparece como capacidade opcional indisponível, não como erro do core.
4. Fixture pública/local termina com React/TS build, relatório versionado e status `concept_ready` ou `prototype_ready` justificado; saída lista checks realmente executados.
5. Um teste de prompt injection escondido no site não consegue alterar instruções, ler segredos nem autorizar writes.
6. Cursor, Claude Code e Codex realizam, cada qual, smoke test do fluxo pelo menos até Study/Check antes de publicação da compatibilidade.
7. Documentação tem um exemplo mínimo sem API paga, mas não promete que modelo local pequeno conseguirá concluir uma tarefa complexa.
8. Um MCP host-agent não inicia `rebuild_start`, um segundo builder ou modelo por conta própria; transferir controle exige confirmação explícita.
9. Sem métricas de tokens do agente hospedeiro, relatórios exibem `unavailable`, nunca consumo zero, nem alegam hard cap no plano de assinatura.
10. Instrumentos de limitação de REA/MCP/critic e tamanho de respostas são verificáveis e regressão de orçamento é testada.

## Guia de adoção

- **Primeira versão:** instruções no README e no `AGENTS.md`, CLI herdada e MCP do Craft. Sem criar tecnologia nova de agente.
- **Após estabilizar:** um arquivo Skill portátil e setups próprios por agente, sempre opcionais.
- **Experiência de produto:** um usuário não deveria precisar conhecer a arquitetura Craft/REA/Layout para completar a tarefa.

## Regras normativas v1.3.0

O agente deve ler `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md` e `SAFE_WORKSPACE_POLICY.md` antes de acessar aplicação privada, executar dependências de terceiros ou aplicar mudanças. `doctor` não executa scripts nem instala dependências. No modo `host_agent`, as ferramentas REA não controlam ações shell externas do host; permissões do Cursor/Claude/Codex devem ser configuradas separadamente. Saída padrão isolada e aplicação de patch apenas com aprovação explícita. Nenhum exemplo de comando proposto é alegado como já implementado.

## Alinhamento v1.4.0 — V1 Impactante

A primeira entrega pública não é MVP. Todas as capacidades e critérios de lançamento estão em `V1_IMPACTANTE.md`. Os checkpoints técnicos internos não reduzem o escopo do release; políticas de segurança, custo e oráculos permanecem obrigatórias.
