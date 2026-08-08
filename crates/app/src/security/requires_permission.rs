//! A permissão que um caso de uso exige.

use crate::context::UserContext;
use crate::error::AppError;

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
    pub(crate) const fn new(slug: &'static str) -> Self {
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
    use crate::security::permission_catalog::PermissionCatalog;
    use crate::security::permission_slug::PermissionSlug;
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
        let mut unicos: Vec<&str> = PermissionCatalog::ALL.to_vec();
        unicos.sort_unstable();
        unicos.dedup();

        assert_eq!(
            unicos.len(),
            PermissionCatalog::ALL.len(),
            "um slug duplicado no catálogo esconde uma permissão que ninguém registrou"
        );
    }

    #[test]
    fn o_catalogo_tem_as_25_permissoes_do_php() {
        // O número é contrato: são as permissões que já existem em papéis
        // gravados. Este teste quebra tanto se alguém acrescentar um caso de uso
        // sem catalogar a permissão quanto se remover uma que ainda está em uso.
        assert_eq!(PermissionCatalog::ALL.len(), 25);
    }

    #[test]
    fn todo_slug_segue_o_formato_recurso_acao() {
        // O TableModule de permissão recusa slug fora deste formato — melhor
        // descobrir aqui do que ver o boot falhar.
        for slug in PermissionCatalog::ALL {
            let (resource, action) = slug
                .split_once(':')
                .unwrap_or_else(|| panic!("slug sem `:`: {slug}"));

            assert!(!resource.is_empty(), "recurso vazio em {slug}");
            assert!(!action.is_empty(), "ação vazia em {slug}");
        }
    }

    #[test]
    fn quem_tem_a_permissao_passa() {
        let guard = RequiresPermission::new(PermissionSlug::PRODUCT_CREATE);

        assert!(guard
            .authorize(&user_with(&[PermissionSlug::PRODUCT_CREATE]))
            .is_ok());
    }

    #[test]
    fn quem_nao_tem_e_recusado_com_o_slug_no_erro() {
        let guard = RequiresPermission::new(PermissionSlug::PRODUCT_CREATE);

        let error = guard
            .authorize(&user_with(&[PermissionSlug::PRODUCT_READ]))
            .expect_err("deveria recusar");

        assert!(matches!(
            error,
            AppError::Forbidden { ref permission } if permission == PermissionSlug::PRODUCT_CREATE
        ));
    }
}
