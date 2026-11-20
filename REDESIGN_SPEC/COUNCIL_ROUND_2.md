# Council II — Revisão adversarial da spec v1.2.0

**Data:** 19/11/2026. **Natureza:** simulação estruturada de oito perspectivas profissionais, não participação de especialistas reais ou de modelos independentes. **Evidência disponível:** documentação/spec e referências aos repositórios; nenhum benchmark integrado foi executado.

## Participantes

1. Arquitetura de software: modularidade, compatibilidade upstream, correção do fork.
2. Engenharia de IA: baselines, inferência, avaliações e controle de contexto.
3. Design/HCI: tarefas, acessibilidade, conteúdo, pesquisa e qualidade operacional.
4. Product Management: posicionamento, escopo, adoção, métricas e estratégia.
5. Segurança e privacidade (novo): credenciais, isolamento, supply chain, MCP e autorização.
6. QA e confiabilidade (novo): oráculos, falsos positivos, regressões e reprodutibilidade.
7. Developer Experience e OSS (novo): instalação, CI, suporte, documentação, licenças e distribuição.
8. Especialista em operação empresarial (novo): papéis, transações, processos, dados e impactos reais.

## Rodada adversarial — críticas e respostas

### Arquitetura
**Objeção:** Mesmo o v1.2 mantém contratos e nomes de módulos que podem não mapear de forma estável aos pontos de extensão do Craft. `TargetKind` sozinho não garante CLI/MCP/desktop, persistência e gating. **Resposta:** Slice 0 precisa fixar SHA e mapear integrações reais; Slice 1 só passa com E2E headless. **Veto:** alterações profundas no core sem ADR comparando fork e extensão.

### IA
**Objeção:** A/B/C não são comparáveis se B recebe instruções piores do que C, se o modelo muda ou se os avaliadores conhecem a origem. **Resposta:** rubrica pré-registrada, mesmo orçamento e prompt B forte, avaliação cega, múltiplas execuções e registro de variância. **Veto:** reivindicação de superioridade baseada só em LLM-as-judge.

### Design/HCI
**Objeção:** `Task Model` é necessário, mas frequência/criticidade frequentemente são desconhecidas. Uma tela mais limpa pode ser pior para usuários experientes. **Resposta:** `unknown` legítimo; preservar alternativa original; incluir teclado, densidade, tarefas em lote, estados e conteúdo. **Veto:** aprovar redesign por estética, screenshots ou score heurístico isolado.

### Produto
**Objeção:** “Qualquer aplicação web” é ambicioso demais; valor para designer e para modernizador são jobs distintos. **Resposta:** primeiro usuário-alvo = desenvolvedor ou designer técnico que precisa redesenhar uma jornada web existente com código verificável. Produto de modernização integral é horizonte, não promessa P0. **Veto:** expansão de escopo antes de prova de diferenciação.

### Segurança e privacidade
**Objeção:** `host_agent` é controlado por outro processo; o REA não consegue garantir por si só que o agente externo não executará comandos ou enviará capturas a um modelo remoto. **Resposta:** separar controles *enforceable by REA* de orientações ao host; não declarar sandbox completo; isolamento por projeto, autorização explícita e revisão de dados. **Veto:** execução de scripts de origem não confiável, exportação de cookies/tokens ou escrita em produção sem aprovação.

### QA e confiabilidade
**Objeção:** `Prototype Ready` pode ser falsamente aprovado se os testes cobrem apenas os fluxos conhecidos e o relatório omite os desconhecidos. **Resposta:** gate exige denominador e escopo explícitos; resultado de check com erro/timeout = falha/inconclusivo; oráculos independentes das hipóteses do próprio gerador. **Veto:** qualquer `passed=true` sem execução verificável.

### DX/OSS
**Objeção:** A experiência “passe a URL” não será igual em Cursor, Claude e Codex se exigir Rust, Node, browsers, permissões e configuração de MCP. **Resposta:** bootstrap por etapas com `doctor`, caminhos alternativos e setup explícito; testar ambiente limpo por agente/SO; distinguir licença do core e licenças de cada integração. **Veto:** alegar instalação automática universal ou custo zero de inferência.

### Operação empresarial
**Objeção:** Consolidar cinco telas pode exigir atomicidade, auditoria e permissões específicas. Um protótipo com mock não demonstra equivalência de negócio. **Resposta:** classificar mudanças UI-only/orchestration/backend-required; usar MCP semântico apenas como fonte, com verificação cruzada; evidenciar regras desconhecidas e proibir inferência de transações. **Veto:** `Engineering Ready` para fluxos críticos sem contrato ou oráculo confiável.

## Debates entre membros

**Produto vs. HCI:** Produto defende entregar uma única tela rapidamente; HCI exige pelo menos uma tarefa completa. **Conciliação:** uma jornada pequena, não uma screenshot isolada.

**IA vs. Arquitetura:** IA quer manter o Design Engine flexível; Arquitetura quer contratos estáveis. **Conciliação:** contratos para evidências e resultados, não para cada passo interno de raciocínio.

**DX vs. Segurança:** DX quer bootstrap sem atrito; Segurança veta scripts automáticos. **Conciliação:** diagnóstico automático, execução de instalações apenas com consentimento e registro.

**QA vs. Produto:** Produto quer release rápido; QA exige oráculos independentes. **Conciliação:** Concept Ready pode sair com hipóteses; Prototype Ready exige cenários críticos conhecidos testados; Engineering Ready exige integração e escopo explícito.

## Achados novos — lacunas ainda abertas na v1.2.0

1. **Enforcement fora do processo:** a ferramenta não controla todos os comandos do agente hospedeiro; precisa documentação de fronteiras de confiança e testes host-specific.
2. **Benchmark sem protocolo de amostragem fechado:** quantidade de fixtures/repetições e limiares devem ser pré-registrados antes de declarar ganho.
3. **Fonte de verdade de oráculos:** não gerar testes e expected results exclusivamente da própria hipótese do redesign; preferir comportamento observado, contrato ou confirmação humana.
4. **Edição de projeto existente vs. geração nova:** risco de sobrepor código e rotas do cliente; P0 deve gerar em workspace separado e exigir aprovação para aplicar patches.
5. **Reprodutibilidade de design systems privados:** pacotes, tokens e Storybook podem mudar; registrar hash/versão, evitar redistribuição de ativos proprietários.
6. **Compatibilidade do fork com upstream:** estabelecer política de cherry-pick/merge, CI de regressão e ADR se o delta crescer.
7. **Acessibilidade além do axe:** keyboard-only, foco, leitor de tela em amostra e alto zoom; declarar limitações.
8. **Segurança de prompts e evidências:** fontes não confiáveis, retenção, TTL, redaction e proibição de exportar sessões.

## Votação simulada

- Arquitetura: **aprova condicionalmente** ao spike e isolamento do core.
- IA: **aprova condicionalmente** ao benchmark contra baseline B forte.
- Design/HCI: **aprova condicionalmente** ao Task Model e teste por jornada.
- Produto: **aprova condicionalmente** ao posicionamento e P0 restrito.
- Segurança: **aprova condicionalmente** a controles de autorização, dados e supply chain.
- QA: **aprova condicionalmente** a oráculos independentes e falha fechada.
- DX/OSS: **aprova condicionalmente** a bootstrap honesto e CI multiambiente.
- Domínio empresarial: **aprova condicionalmente** a limites transacionais explícitos.

**Decisão: GO para Slice 0 e Slice 1; NO-GO para alegações de paridade completa, uso corporativo sensível sem controles testados ou release Engineering Ready sem evidência.** Não há nota empírica de qualidade nem estimativa confiável de ganho de UX.

## Alterações recomendadas para a próxima revisão

1. Criar `TRUST_BOUNDARIES.md` com controles reais do processo REA vs. responsabilidades do agente host.
2. Criar `ORACLE_POLICY.md` e exigir independência de expected results.
3. Registrar política de output separado e aplicação de patches mediante aprovação.
4. Fixar protocolo de benchmark antes da execução, inclusive amostragem e limiares.
5. Criar checklist de DX e licenças por sistema operacional e agente.

Essas alterações são recomendações; não foram implementadas nem testadas nesta rodada.
