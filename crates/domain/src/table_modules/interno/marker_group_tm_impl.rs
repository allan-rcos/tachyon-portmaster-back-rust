//! A implementação das regras de grupo de marcador.

use crate::error::MetadataError;
use crate::models::interno::marker_group_model::MarkerGroupModel;
use crate::models::MarkerGroup;
use crate::table_modules::interno::slug::Slug;
use crate::table_modules::MarkerGroupTM;

/// A implementação.
#[derive(Clone)]
pub(crate) struct MarkerGroupTMImpl;

impl MarkerGroupTMImpl {
    /// Monta o `TableModule`.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl MarkerGroupTM for MarkerGroupTMImpl {
    fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError> {
        let valid = Slug::is_kebab_token(&slug);
        Slug::validate(
            &slug,
            valid,
            "Slug must be a lower-kebab token (e.g. refresh-token).",
        )?;

        Ok(Box::new(MarkerGroupModel::new(slug)))
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
