# Safe Workspace & Patch Policy — v1.3.0

## Regra padrão
Gerar redesign em workspace **separado** (`output_mode=isolated`) e preservar código-fonte, rotas, arquivos de configuração e segredos do projeto-alvo. Não usar `git reset --hard`, limpeza destrutiva, sobrescrita ou auto-merge.

## Fluxo de aplicação
1. Detectar estado do repositório, branch, alterações não commitadas e arquivos afetados.
2. Gerar artefatos em diretório isolado e registrar hashes de entradas/saídas.
3. Construir um diff legível com rotas, imports, APIs, dados e dependências alteradas.
4. Classificar impacto: `ui_only`, `frontend_orchestration`, `backend_change_required`.
5. Executar testes conhecidos, scans de segredos e check de conflitos.
6. Solicitar aprovação humana explícita por escopo e alterações antes de aplicar.
7. Aplicar atomicamente quando possível; se conflito, abortar sem descartar alterações locais.
8. Registrar rollback documentado; nunca declarar rollback seguro sem teste.

## Exceções
Modo `in_place` somente por opt-in explícito e confirmação adicional. `backend_change_required` não cria contratos ou endpoints fictícios. `apply_patch` não é habilitado em ambiente de produção sem autorização específica e avaliação de risco.

## Testes bloqueantes
SW-01 dirty tree: não sobrescreve arquivos.
SW-02 rota/API existente alterada: diff e aprovação obrigatórios.
SW-03 conflito entre geração e aplicação: aborta sem perda.
SW-04 symlink para fora da raiz: recusa.
SW-05 patch sem aprovação: recusa.
SW-06 build/test falha: bloqueia aplicação por padrão.
SW-07 pedido de transação atômica sem backend: `backend_change_required`.
**Status:** política de design, não garantia implementada.
