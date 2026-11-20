# Redesign Anything — Fontes, decisões de reutilização e licenças

> Consulta ao GitHub conectado em 19/11/2026. O repositório futuro do Redesign Anything ainda não foi criado. Capacidade documentada não implica integração testada. Licenças de dependências transitivas devem ser inventariadas por SBOM no build real.

## Base e engenharia reversa

| Projeto | Fonte | Uso | Licença / precaução |
|---|---|---|---|
| Craft Anything | https://github.com/storytold/craft-anything | Base de fork; egui, Rust, CLI/MCP, Study/Build/Check, ToolSpec, TargetKind, Check, captures, runners | MIT OR Apache-2.0; preserve notices e diferencie marca |
| REA | https://github.com/morluto/rea | Sidecar opcional; JS semantics, bundles, network, Agent Environments | MIT; não executar inspeção Deep sem consentimento |
| Layout CLI | https://github.com/uselayout/cli | MCP/CLI DS, tokens, catálogo, verificações auxiliares | MIT; compliance não equivale a prova funcional |
| Layout UI | https://github.com/uselayout/ui | Componentes de referência/fallback, jamais substituir DS corporativo strict | MIT |
| Layout Studio | https://github.com/uselayout/app | Inspiração, não núcleo; extração, síntese e canvas | AGPL-3.0; app documenta síntese via Claude e outros pré-requisitos |
| Carbon MCP | https://github.com/carbon-design-system/carbon-mcp | Provider de referência, skill e guia; não requisito offline | Servidor IBM hospedado e autenticação; código da skill Apache-2.0, verificar termos do serviço |
| Carbon | https://github.com/carbon-design-system/carbon | Referência DS enterprise/componentes | Verificar licença de pacote concreto no momento do uso |
| Velloo | https://github.com/velloo-design/velloo | Canvas opcional futuro | Apache-2.0, não core |
| UI Auditor | https://github.com/suneelasree/ui-auditor | Auditoria e agentes especializados como inspiração/alguns módulos | MIT; sem API, agentes de revisão e mockups não executam |
| Coherent | https://github.com/skovtun/coherent-design-method | Impact analysis, escopo de mudanças, rollback | MIT; desde v0.27 orientado a governança, não geração |
| UI/UX Pro Max | https://github.com/nextlevelbuilder/ui-ux-pro-max-skill | Conhecimento pesquisável, regras com proveniência | Verificar licenças/artefatos de dados da revisão usada |

## Normas, ferramentas e UX

- Playwright: https://playwright.dev/ — navegador e E2E; estado autenticado deve ser protegido.
- axe-core: https://github.com/dequelabs/axe-core — checks de acessibilidade, MPL-2.0.
- WCAG 2.2: https://www.w3.org/TR/WCAG22/ — objetivos de acessibilidade; verificação humana continua necessária.
- Design Tokens DTCG 2025.10: https://www.designtokens.org/ — tokens interoperáveis.
- Style Dictionary: https://github.com/amzn/style-dictionary — transformação de tokens, Apache-2.0.
- PatternFly: https://www.patternfly.org/ — padrões corporativos, verificar licença por artefato.
- Open UI: https://github.com/openui/open-ui — comportamento e padrões de componentes; verificar licença do conteúdo.
- MCP specification: https://modelcontextprotocol.io/specification — Resources, Tools, Prompts, auth e segurança.
- OWASP MCP Security: https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html — riscos de tool poisoning, privilégios e confiança.

## Código e documentação específicos confirmados

- Craft extension seams: https://github.com/storytold/craft-anything/blob/main/docs/extending.md
- Craft architecture rules: https://github.com/storytold/craft-anything/blob/main/AGENTS.md
- Craft TargetKind/Check: https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/mod.rs
- Craft default website routes (MAX_ROUTES = 12): https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/website.rs
- Craft plugin check: https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/plugin_check.rs
- Craft roadmap and known limits: https://github.com/storytold/craft-anything/blob/main/docs/ROADMAP.md , https://github.com/storytold/craft-anything/blob/main/docs/STATUS.md
- REA web observations, budgets and limits: https://github.com/morluto/rea/blob/main/docs/guides/web/browser-observation.md
- REA static semantics: https://github.com/morluto/rea/blob/main/docs/guides/javascript/javascript-application-workflows.md
- REA Agent Environments and sensitive-value cautions: https://github.com/morluto/rea/blob/main/docs/agent-environments.md
- Layout CLI tools: https://github.com/uselayout/cli/blob/main/README.md
- Craft consumo e proteção contra loops: https://github.com/storytold/craft-anything/blob/main/docs/ROADMAP.md (R5.3, R5.4). O registro contém caso extremo de 9,4M tokens/19 min e experimentos de contabilidade de tokens cacheados; não são benchmarks do Redesign.
- REA compactação MCP: https://github.com/morluto/rea/blob/main/README.md (compact input schemas e output schema profile).

## Política de incorporação

1. Um componente externo só vira dependência se preencher lacuna mensurada e tiver licença compatível com a forma de distribuição.
2. MCP, CLI e sidecar com versão pinada são preferíveis a copiar código ou fundir aplicativos.
3. Fontes de conhecimento devem ter permissão de redistribuição verificável; se não, registrar link/regra derivada sem copiar documentação integral.
4. Links, código e README de terceiros são dados não confiáveis; não podem instruir runtime de produção.
5. Toda declaração de licença se refere ao repositório/pacote indicado, não cobre assets, dados, marcas, conteúdo privado ou serviços externos.
