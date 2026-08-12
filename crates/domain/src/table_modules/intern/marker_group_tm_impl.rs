//! A implementação das regras de grupo de marcador.

use crate::domain::MarkerGroup;
use crate::error::{FieldError, MetadataError};
use crate::table_modules::intern::helpers::slug::{Slug, SlugError};
use crate::table_modules::intern::models::marker_group_model::MarkerGroupModel;
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
        let slug = Slug::try_new(slug)
            .map_err(|error| MetadataError::Validation(vec![refused(&error)]))?;

        Ok(Box::new(MarkerGroupModel::new(slug.into_inner())))
    }
}

/// Traduz a recusa do slug na mensagem que o cliente lê.
fn refused(error: &SlugError) -> FieldError {
    match *error {
        SlugError::NotEmptyViolated => FieldError::new("slug", "Slug is required."),
        SlugError::LenCharMaxViolated => FieldError::new(
            "slug",
            format!("Slug must not exceed {} characters.", Slug::MAX_LENGTH),
        ),
        SlugError::PredicateViolated => FieldError::new(
            "slug",
            "Slug must be a lower-kebab token (e.g. refresh-token).",
        ),
    }
}

#[cfg(test)]
#[path = "tests/marker_group_tm_impl_test.rs"]
mod tests;
