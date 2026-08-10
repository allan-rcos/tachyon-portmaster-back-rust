//! A implementação das regras de permissão.

use nutype::nutype;

use crate::domain::Permission;
use crate::error::{FieldError, MetadataError};
use crate::table_modules::intern::helpers::slug::Slug;
use crate::table_modules::intern::models::permission_model::PermissionModel;
use crate::table_modules::PermissionTM;

/// O slug de uma permissão: `domain:action`, cada lado um token kebab.
///
/// O comprimento é conferido no todo, e não em cada metade: são as duas juntas
/// que viram a chave lida a cada verificação de acesso.
#[nutype(validate(not_empty, len_char_max = 64, predicate = is_domain_action))]
struct PermissionSlug(String);

/// A implementação. Sem helpers: é validação de formato e nada mais.
#[derive(Clone)]
pub(crate) struct PermissionTMImpl;

impl PermissionTMImpl {
    /// Monta o `TableModule`.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl PermissionTM for PermissionTMImpl {
    fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError> {
        let slug = PermissionSlug::try_new(slug)
            .map_err(|error| MetadataError::Validation(vec![refused(&error)]))?;

        Ok(Box::new(PermissionModel::new(slug.into_inner())))
    }
}

/// Se o valor é `domain:action`, com os dois lados em lower-kebab.
fn is_domain_action(value: &str) -> bool {
    let mut parts = value.split(':');

    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(domain), Some(action), None)
            if Slug::is_kebab_token(domain) && Slug::is_kebab_token(action)
    )
}

/// Traduz a recusa do slug na mensagem que o cliente lê.
fn refused(error: &PermissionSlugError) -> FieldError {
    match *error {
        PermissionSlugError::NotEmptyViolated => FieldError::new("slug", "Slug is required."),
        PermissionSlugError::LenCharMaxViolated => FieldError::new(
            "slug",
            format!("Slug must not exceed {} characters.", Slug::MAX_LENGTH),
        ),
        PermissionSlugError::PredicateViolated => FieldError::new(
            "slug",
            r#"Slug must follow "domain:action" in lower-kebab (e.g. product:create)."#,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn aceita_os_slugs_que_os_casos_de_uso_declaram() {
        for good in [
            "product:create",
            "container:seal",
            "manifest:load",
            "user:update-roles",
            "role:update-permissions",
            "metrics:read",
        ] {
            let permission = PermissionTMImpl::new()
                .create(good.into())
                .unwrap_or_else(|_| panic!("{good} deveria ser aceito"));
            assert_eq!(permission.slug(), good);
        }
    }

    #[test]
    fn recusa_slug_fora_do_formato() {
        for bad in [
            "",
            "sem-dois-pontos",
            "product:",
            ":create",
            "Product:Create",
            "product:create:extra",
            "product::create",
        ] {
            assert!(
                PermissionTMImpl::new().create(bad.into()).is_err(),
                "deveria recusar {bad:?}"
            );
        }
    }

    #[test]
    fn recusa_slug_longo_demais() {
        let long = format!("{}:{}", "a".repeat(40), "b".repeat(40));
        assert!(PermissionTMImpl::new().create(long).is_err());
    }
}
