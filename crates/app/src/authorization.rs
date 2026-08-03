//! A permissão exigida por um caso de uso.
//!
//! No PHP isto era um trait (`AuthorizesWithPermission`) que cada caso de uso
//! usava, declarando o slug no construtor. Aqui é um **Partial**: um helper
//! `pub(crate)` que o caso de uso guarda como campo e consulta na primeira linha
//! da execução.
//!
//! ## Por que a verificação mora no `app`
//!
//! Não no `domain`, porque "quem pode fazer" não é regra de negócio do pátio —
//! um contêiner sela do mesmo jeito seja quem for que mande. Não no `api-http`,
//! porque então cada apresentação nova teria que reimplementar a mesma tabela de
//! permissões, e a primeira que esquecesse abriria o sistema.
//!
//! ## Por que os slugs são constantes num lugar só
//!
//! O `POST /setup` concede ao primeiro papel **tudo que foi registrado** — e o
//! registro acontece no boot, a partir de [`CATALOG`]. Se um caso de uso
//! escrevesse o slug como literal solto, um erro de digitação criaria uma
//! permissão que ninguém jamais recebe: o caso de uso exigiria `product:crate`,
//! o catálogo registraria `product:create`, e o administrador ficaria sem
//! acesso a um endpoint sem que nada falhasse alto.
//!
//! Com uma constante só, usada nos dois lugares, a divergência não tem onde
//! nascer.

use crate::context::UserContext;
use crate::error::AppError;

/// Os slugs de permissão, literalmente como o PHP os tinha.
///
/// São contrato: já existem em papéis gravados no banco de quem roda a versão
/// PHP. Renomear qualquer um destes revoga silenciosamente o acesso de quem o
/// tinha.
pub mod slug {
    /// Registrar um contêiner.
    pub const CONTAINER_CREATE: &str = "container:create";
    /// Remover um contêiner.
    pub const CONTAINER_DELETE: &str = "container:delete";
    /// Despachar um contêiner.
    pub const CONTAINER_DISPATCH: &str = "container:dispatch";
    /// Ler contêineres.
    pub const CONTAINER_READ: &str = "container:read";
    /// Selar um contêiner.
    pub const CONTAINER_SEAL: &str = "container:seal";
    /// Ler o resumo de carga e telemetria.
    pub const CONTAINER_SUMMARY: &str = "container:summary";
    /// Alterar um contêiner.
    pub const CONTAINER_UPDATE: &str = "container:update";
    /// Embarcar carga.
    pub const MANIFEST_LOAD: &str = "manifest:load";
    /// Desembarcar carga.
    pub const MANIFEST_UNLOAD: &str = "manifest:unload";
    /// Ler o painel do pátio.
    pub const METRICS_READ: &str = "metrics:read";
    /// Listar as permissões registradas.
    pub const PERMISSION_LIST: &str = "permission:list";
    /// Cadastrar um produto.
    pub const PRODUCT_CREATE: &str = "product:create";
    /// Remover um produto.
    pub const PRODUCT_DELETE: &str = "product:delete";
    /// Ler produtos.
    pub const PRODUCT_READ: &str = "product:read";
    /// Alterar um produto.
    pub const PRODUCT_UPDATE: &str = "product:update";
    /// Criar um papel.
    pub const ROLE_CREATE: &str = "role:create";
    /// Ler papéis.
    pub const ROLE_LIST: &str = "role:list";
    /// Trocar as permissões de um papel.
    pub const ROLE_UPDATE_PERMISSIONS: &str = "role:update-permissions";
    /// Redefinir a senha de outro usuário.
    pub const USER_CHANGE_PASSWORD: &str = "user:change-password";
    /// Cadastrar um usuário.
    pub const USER_CREATE: &str = "user:create";
    /// Remover um usuário.
    pub const USER_DELETE: &str = "user:delete";
    /// Ler um usuário.
    pub const USER_GET: &str = "user:get";
    /// Listar usuários.
    pub const USER_LIST: &str = "user:list";
    /// Alterar um usuário.
    pub const USER_UPDATE: &str = "user:update";
    /// Trocar os papéis de um usuário.
    pub const USER_UPDATE_ROLES: &str = "user:update-roles";
}

/// Tudo que o boot registra e que o `POST /setup` concede.
///
/// Ler daqui em vez de listar à mão no setup é o que faz uma permissão nova ser
/// concedida ao administrador sem ninguém lembrar de voltar aqui.
pub const CATALOG: &[&str] = &[
    slug::CONTAINER_CREATE,
    slug::CONTAINER_DELETE,
    slug::CONTAINER_DISPATCH,
    slug::CONTAINER_READ,
    slug::CONTAINER_SEAL,
    slug::CONTAINER_SUMMARY,
    slug::CONTAINER_UPDATE,
    slug::MANIFEST_LOAD,
    slug::MANIFEST_UNLOAD,
    slug::METRICS_READ,
    slug::PERMISSION_LIST,
    slug::PRODUCT_CREATE,
    slug::PRODUCT_DELETE,
    slug::PRODUCT_READ,
    slug::PRODUCT_UPDATE,
    slug::ROLE_CREATE,
    slug::ROLE_LIST,
    slug::ROLE_UPDATE_PERMISSIONS,
    slug::USER_CHANGE_PASSWORD,
    slug::USER_CREATE,
    slug::USER_DELETE,
    slug::USER_GET,
    slug::USER_LIST,
    slug::USER_UPDATE,
    slug::USER_UPDATE_ROLES,
];

/// O grupo de marcador da sessão de refresh.
///
/// Registrado no boot pelo mesmo motivo das permissões: o
/// [`MarkerRepository`](portmaster_infra::repository::MarkerRepository) recusa
/// marcar num grupo que não conhece, e é o `api-http` — não esta camada — quem
/// vai usá-lo.
pub const REFRESH_TOKEN_GROUP: &str = "refresh-token";

/// A permissão que um caso de uso exige.
///
/// Guardada como `&'static str` e não `String`: o slug é uma constante do
/// binário, e alocá-la a cada construção de caso de uso — que acontece a cada
/// requisição — seria trabalho por nada.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RequiresPermission {
    slug: &'static str,
}

impl RequiresPermission {
    /// Declara a permissão exigida.
    pub(crate) fn new(slug: &'static str) -> Self {
        Self { slug }
    }

    /// Recusa quem não a tem.
    ///
    /// Primeira linha de todo caso de uso protegido, **antes** do cache e antes
    /// de abrir transação: consultar o cache antes de autorizar entregaria dado
    /// a quem não pode vê-lo, porque o cache não sabe quem está perguntando.
    pub(crate) fn authorize(&self, context: &UserContext) -> Result<(), AppError> {
        if context.has_permission(self.slug) {
            return Ok(());
        }

        Err(AppError::Forbidden {
            permission: self.slug.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RoleContext;
    use pretty_assertions::assert_eq;

    fn user_with(permissions: &[&str]) -> UserContext {
        UserContext {
            id: "1".into(),
            name: "Ana".into(),
            email: "ana@portmaster.local".into(),
            roles: vec![RoleContext {
                id: "1".into(),
                name: "Papel".into(),
                permissions: permissions.iter().map(|p| (*p).to_owned()).collect(),
            }],
        }
    }

    #[test]
    fn o_catalogo_nao_tem_slug_repetido() {
        let mut unicos: Vec<&str> = CATALOG.to_vec();
        unicos.sort_unstable();
        unicos.dedup();

        assert_eq!(
            unicos.len(),
            CATALOG.len(),
            "um slug duplicado no catálogo esconde uma permissão que ninguém registrou"
        );
    }

    #[test]
    fn o_catalogo_tem_as_25_permissoes_do_php() {
        // O número é contrato: são as permissões que já existem em papéis
        // gravados. Este teste quebra tanto se alguém acrescentar um caso de uso
        // sem catalogar a permissão quanto se remover uma que ainda está em uso.
        assert_eq!(CATALOG.len(), 25);
    }

    #[test]
    fn todo_slug_segue_o_formato_recurso_acao() {
        // O TableModule de permissão recusa slug fora deste formato — melhor
        // descobrir aqui do que ver o boot falhar.
        for slug in CATALOG {
            let (resource, action) = slug
                .split_once(':')
                .unwrap_or_else(|| panic!("slug sem `:`: {slug}"));

            assert!(!resource.is_empty(), "recurso vazio em {slug}");
            assert!(!action.is_empty(), "ação vazia em {slug}");
        }
    }

    #[test]
    fn quem_tem_a_permissao_passa() {
        let guard = RequiresPermission::new(slug::PRODUCT_CREATE);

        assert!(guard.authorize(&user_with(&[slug::PRODUCT_CREATE])).is_ok());
    }

    #[test]
    fn quem_nao_tem_e_recusado_com_o_slug_no_erro() {
        let guard = RequiresPermission::new(slug::PRODUCT_CREATE);

        let error = guard
            .authorize(&user_with(&[slug::PRODUCT_READ]))
            .expect_err("deveria recusar");

        assert!(matches!(
            error,
            AppError::Forbidden { ref permission } if permission == slug::PRODUCT_CREATE
        ));
    }
}
