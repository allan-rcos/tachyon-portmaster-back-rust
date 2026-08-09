//! A implementação das regras de marcador.

use crate::error::{MarkerError, Validation};
use crate::models::interno::marker_model::MarkerModel;
use crate::models::Marker;
use crate::security::IndexHasher;
use crate::table_modules::interno::slug::Slug;
use crate::table_modules::MarkerTM;

/// A implementação, genérica sobre o hasher de indexação.
#[derive(Clone)]
pub(crate) struct MarkerTMImpl<H> {
    /// Quem transforma a chave do marcador em índice — rápido de propósito.
    hasher: H,
}

impl<H: IndexHasher> MarkerTMImpl<H> {
    /// Monta o `TableModule` com o seu hasher.
    pub(crate) const fn new(hasher: H) -> Self {
        Self { hasher }
    }
}

impl<H: IndexHasher> MarkerTM for MarkerTMImpl<H> {
    /// Monta um marcador, com a chave já hasheada.
    ///
    /// Valor vazio é recusado, e não é preciosismo: ele hasharia para uma
    /// constante, e aí todo chamador que esquecesse de passar algo
    /// compartilharia um único marcador — cada um vendo o booleano dos outros
    /// virar.
    fn create(
        &self,
        group: String,
        plain: &str,
        flag: bool,
    ) -> Result<Box<dyn Marker>, MarkerError> {
        let mut errors = Validation::new();

        if group.is_empty() {
            errors.add("group", "Group is required.");
        } else if !Slug::is_kebab_token(&group) {
            errors.add(
                "group",
                "Group must be a lower-kebab token (e.g. refresh-token).",
            );
        }

        errors.add_if(plain.is_empty(), "value", "Value is required.");

        errors.into_result(()).map_err(MarkerError::Validation)?;

        Ok(Box::new(MarkerModel::new(
            group,
            self.hasher.hash(plain),
            flag,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::interno::xx_index_hasher::XxIndexHasher;
    use pretty_assertions::assert_eq;

    fn table_module() -> MarkerTMImpl<XxIndexHasher> {
        MarkerTMImpl::new(XxIndexHasher::new())
    }

    #[test]
    fn guarda_o_digest_e_nao_o_valor() {
        let marker = table_module()
            .create("refresh-token".into(), "token-secreto", true)
            .expect("grupo e valor são válidos");

        assert_eq!(marker.group(), "refresh-token");
        assert_ne!(marker.key(), "token-secreto");
        assert!(marker.flag());
    }

    /// É o que faz o refresh funcionar.
    ///
    /// Marcar no login e consultar depois precisam cair na mesma chave.
    #[test]
    fn o_mesmo_valor_reencontra_a_mesma_marca() {
        let first = table_module()
            .create("refresh-token".into(), "abc", true)
            .expect("válido");
        let second = table_module()
            .create("refresh-token".into(), "abc", false)
            .expect("válido");

        assert_eq!(first.key(), second.key());
    }

    #[test]
    fn recusa_valor_vazio() {
        let error = table_module()
            .create("refresh-token".into(), "", true)
            .err()
            .expect("valor vazio colidiria todo mundo numa marca só");

        let MarkerError::Validation(fields) = error;
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "value");
    }

    #[test]
    fn recusa_grupo_fora_do_formato() {
        for bad in ["", "Refresh-Token", "refresh_token"] {
            assert!(
                table_module().create(bad.into(), "abc", true).is_err(),
                "deveria recusar grupo {bad:?}"
            );
        }
    }
}
