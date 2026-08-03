//! A permissão: o que um caso de uso exige para ser executado.

use crate::error::MetadataError;

use super::{is_kebab_token, validate_slug};

/// Uma capacidade que um papel pode conceder.
///
/// O slug é tudo que uma permissão é. Não há rótulo nem descrição: o catálogo é
/// o próprio código — cada caso de uso declara a sua no boot — e um texto de
/// exibição seria uma segunda fonte de verdade a manter sincronizada com ele.
pub trait Permission: Send + Sync {
    /// O slug, em `domain:action`.
    fn slug(&self) -> &str;
}

/// A implementação do domínio de [`Permission`].
pub(crate) struct PermissionModel {
    slug: String,
}

impl Permission for PermissionModel {
    fn slug(&self) -> &str {
        &self.slug
    }
}

/// Constrói permissões, recusando slug fora do formato.
pub trait PermissionTM {
    /// Cria a permissão de um slug.
    fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError>;
}

/// A implementação. Sem helpers: é validação de formato e nada mais.
pub(crate) struct PermissionTMImpl;

impl PermissionTMImpl {
    /// Monta o TableModule.
    pub(crate) fn new() -> Self {
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
                if is_kebab_token(domain) && is_kebab_token(action)
        );

        validate_slug(
            &slug,
            valid,
            r#"Slug must follow "domain:action" in lower-kebab (e.g. product:create)."#,
        )?;

        Ok(Box::new(PermissionModel { slug }))
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
