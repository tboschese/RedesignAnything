# DX, OSS, dependências e upstream — v1.3.0

## Agent-first
`README` e `AGENTS.md` devem permitir que um agente leia a URL, entenda o projeto e execute `doctor` sem instalações implícitas. Bootstrap é assistido: apresentar plano, dependências, comandos e riscos, solicitar consentimento antes de instalar, compilar ou registrar MCP. Compatibilidade não deve ser anunciada antes de smoke tests reais em Cursor, Claude Code e Codex.

## Matriz de compatibilidade
Registrar para cada SO/agente: versão, autenticação, CLI/MCP, bootstrap, permissões, build, check, retomada, uso de tokens reportado ou unavailable, resultado e limitações. Mínimo para release: Linux/macOS/Windows conforme disponibilidade de CI e testes reais; plataformas não testadas devem ser rotuladas não verificadas.

## Design systems privados
Capturar nome, versão exata, lockfile/digest, origem, hashes de tokens e componentes, licença e permissões de redistribuição. Não copiar Storybook, Figma, assets, fontes ou componentes privados para pacote público. Cache segregado, versão pinada; divergência de hash exige revalidação.

## Fork/upstream
Registrar SHA de origem, licença/attribution e patches locais. Atualização upstream em branch dedicada com CI de regressão, migrations e comparação dos seams (`TargetKind`, `Check`, `ToolSpec`, MCP, CLI). Revisão periódica de CVEs/dependências. ADR obrigatório se delta do core crescer ou API de extensão quebrar; comparar manter fork vs skill/CLI/MCP externos.

## Acessibilidade
Automação (axe, semântica, contraste) **não substitui** teclado, foco, zoom 200%/400% conforme cenário, leitores de tela em amostra e testes com usuários quando necessário. Resultado deve dizer quais modalidades foram testadas.

## Testes
DX-01 doctor não instala nem executa postinstall.
DX-02 instalação solicita consentimento e oferece rollback.
DX-03 versão de DS muda: cache invalidado, checks repetidos.
DX-04 ativo privado não aparece no ZIP/telemetria.
DX-05 merge upstream preserva testes do Craft e redesign.
DX-06 docs não afirmam suporte a plataforma não verificada.
DX-07 teclado, foco e zoom incluídos no plano de verificação.
**Status:** critérios futuros; não verificados em execução real.
