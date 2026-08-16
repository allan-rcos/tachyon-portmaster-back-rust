//! A implementação das regras de marcador.

use crate::domain::Marker;
use crate::error::{FieldError, MarkerError};
use crate::security::IndexHasher;
use crate::table_modules::intern::helpers::slug::{Slug, SlugError};
use crate::table_modules::intern::models::marker_model::MarkerModel;
use crate::table_modules::MarkerTM;

/// Monta as regras de marcador com o seu hasher de indexação.
///
/// O hasher chega injetado e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo.
pub(crate) fn marker_tm<H>(hasher: H) -> impl MarkerTM + Send + Sync + Clone + use<H> + 'static
where
    H: IndexHasher + Send + Sync + Clone + 'static,
{
    MarkerTMImpl { hasher }
}

/// A implementação, genérica sobre o hasher de indexação.
#[derive(Clone)]
struct MarkerTMImpl<H> {
    /// Quem transforma a chave do marcador em índice — rápido de propósito.
    hasher: H,
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
        let checked_group = Slug::try_new(group);

        let mut errors = Vec::new();
        if let Err(error) = &checked_group {
            errors.push(group_refused(error));
        }
        if plain.is_empty() {
            errors.push(FieldError::new("value", "Value is required."));
        }

        let (Ok(group), true) = (checked_group, errors.is_empty()) else {
            return Err(MarkerError::Validation(errors));
        };

        Ok(Box::new(MarkerModel::new(
            group.into_inner(),
            self.hasher.hash(plain),
            flag,
        )))
    }
}

/// Traduz a recusa do grupo na mensagem que o cliente lê.
///
/// Fala de "group" e não de "slug": aqui o grupo é um **campo** do marcador que
/// chegou, e é esse nome que o cliente reconhece no corpo que ele mandou.
fn group_refused(error: &SlugError) -> FieldError {
    match *error {
        SlugError::NotEmptyViolated => FieldError::new("group", "Group is required."),
        SlugError::LenCharMaxViolated => FieldError::new(
            "group",
            format!("Group must not exceed {} characters.", Slug::MAX_LENGTH),
        ),
        SlugError::PredicateViolated => FieldError::new(
            "group",
            "Group must be a lower-kebab token (e.g. refresh-token).",
        ),
    }
}

#[cfg(test)]
#[path = "tests/marker_tm_impl_test.rs"]
mod tests;
