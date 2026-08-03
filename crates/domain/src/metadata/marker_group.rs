//! O grupo de marcador: o espaço de nomes em que uma marca vive.

use crate::error::MetadataError;

use super::{is_kebab_token, validate_slug};

/// Um espaço de nomes de marcadores, registrado no boot.
///
/// Existe para que a `infra` possa recusar uma marca destinada a um grupo que
/// ninguém declarou — o que impede que um erro de digitação crie silenciosamente
/// um espaço de nomes paralelo em que nada é encontrado.
pub trait MarkerGroup: Send + Sync {
    /// O slug, em lower-kebab.
    fn slug(&self) -> &str;
}

/// A implementação do domínio de [`MarkerGroup`].
pub(crate) struct MarkerGroupModel {
    slug: String,
}

impl MarkerGroup for MarkerGroupModel {
    fn slug(&self) -> &str {
        &self.slug
    }
}

/// Constrói grupos de marcador.
pub trait MarkerGroupTM {
    /// Cria o grupo de um slug.
    fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError>;
}

/// A implementação.
pub(crate) struct MarkerGroupTMImpl;

impl MarkerGroupTMImpl {
    /// Monta o TableModule.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl MarkerGroupTM for MarkerGroupTMImpl {
    fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError> {
        let valid = is_kebab_token(&slug);
        validate_slug(
            &slug,
            valid,
            "Slug must be a lower-kebab token (e.g. refresh-token).",
        )?;

        Ok(Box::new(MarkerGroupModel { slug }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn aceita_o_grupo_da_sessao_de_refresh() {
        let group = MarkerGroupTMImpl::new()
            .create("refresh-token".into())
            .expect("o grupo da sessão é válido");

        assert_eq!(group.slug(), "refresh-token");
    }

    #[test]
    fn recusa_slug_fora_do_formato() {
        for bad in ["", "Refresh-Token", "refresh_token", "-refresh", "refresh-"] {
            assert!(
                MarkerGroupTMImpl::new().create(bad.into()).is_err(),
                "deveria recusar {bad:?}"
            );
        }
    }
}
