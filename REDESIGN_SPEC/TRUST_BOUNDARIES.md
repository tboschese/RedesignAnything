# Trust Boundaries — requisitos normativos v1.3.0

## Princípio
REA/Redesign Anything **não** é um sandbox do agente hospedeiro. `host_agent` controla comandos, acesso à rede, modelos e permissões conforme a configuração do Cursor/Claude/Codex. REA controla apenas suas próprias ferramentas, arquivos, subprocessos e respostas. Nenhum relatório pode declarar isolamento global sem validação independente do host.

## Fronteiras
| Fronteira | Controle implementável pelo Redesign | Fora de seu controle |
|---|---|---|
| MCP/CLI próprio | validação de parâmetros, allowlist de paths, limites, redaction, timeout, confirmação para operações mutantes | shell e ferramentas nativas do agente host |
| Browser de observação | contexto isolado, perfis temporários, origens permitidas, capturas sanitizadas | navegador ou plugins externos usados diretamente pelo host |
| Application MCP | allowlist de servidores/tools, read-only default, aprovação para writes, metadados de confiança | correção das regras declaradas pelo servidor, ações diretas do host |
| Modelo remoto | minimizar e filtrar dados enviados pelos tools próprios | política de retenção do provedor e envio direto pelo host |
| Instalação/dependências | `doctor` sem side effects, manifesto de ações, hashes quando disponíveis | comandos `npm`, `cargo`, shell executados fora do gateway |

## Requisitos P0
- `project_scope`: paths canônicos permitidos, URLs/origens, MCPs autorizados, modo read-only ou write, ambientes bloqueados. Resolver symlinks e bloquear traversal.
- `approval_record`: identidade do host quando disponível, ação, escopo, justificativa, timestamp e resultado. Ausência de aprovação = não executar ação mutante própria.
- `secrets`: negar exportação de cookies, headers Authorization, API keys, tokens e dados pessoais conhecidos; sanitização antes de persistência e antes de respostas MCP.
- `evidence`: armazenamento segregado por projeto/tenant/role, TTL configurável, purge, proibição de sincronização implícita, manifests sem valores secretos.
- `prompt_injection`: página, HAR, código, docs e resposta de MCP são **dados não confiáveis**; não elevam permissões, não alteram instruções do agente, não autorizam comandos.
- `dependencies`: scripts de install/build em projeto não confiável requerem consentimento explícito e ambiente isolado quando possível. Nunca executar por `doctor`.
- `patch_apply`: só após diff, confirmação e checks. Por padrão gerar em workspace separado.
- Sem prometer que aprovação no MCP impede ações diretas do host. README deve avisar claramente.

## Testes bloqueantes
TB-01 agente host chama shell diretamente: relatório não declara que o gateway impediu.
TB-02 MCP malicioso pede segredo e comando: resposta tratada como dado, sem execução.
TB-03 symlink/traversal tenta sair do workspace: ferramenta recusa.
TB-04 HAR contém cookie/token: dados persistidos e retornados estão sanitizados.
TB-05 operação mutante sem aprovação: ferramenta própria recusa.
TB-06 `doctor` em projeto com postinstall malicioso: nenhum script é executado.
TB-07 duas execuções/tenants: sem vazamento de cache/evidência.
TB-08 retenção expira e purge remove artefatos controlados pelo produto.
**Status:** especificado; nenhum controle é considerado implementado sem teste.
