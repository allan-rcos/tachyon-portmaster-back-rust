//! A implementação das regras de permissão.

use crate::error::MetadataError;
use crate::models::interno::permission_model::PermissionModel;
use crate::models::Permission;
use crate::table_modules::interno::slug::Slug;
use crate::table_modules::PermissionTM;

/// A implementação. Sem helpers: é validação de formato e nada mais.
pub(crate) struct PermissionTMImpl;

impl PermissionTMImpl {
    /// Monta o `TableModule`.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl PermissionTM for PermissionTMImpl {
    fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError> {
        // `domain:action`, cada lado um token kebab.
        let mut parts = slug.split(':');
        let valid = matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some(domain), Some(action), None)
                if Slug::is_kebab_token(domain) && Slug::is_kebab_token(action)
        );

        Slug::validate(
            &slug,
            valid,
            r#"Slug must follow "domain:action" in lower-kebab (e.g. product:create)."#,
        )?;

        Ok(Box::new(PermissionModel::new(slug)))
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
