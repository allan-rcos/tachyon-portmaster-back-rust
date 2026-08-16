//! A implementação das regras de produto.

use nutype::nutype;

use crate::domain::Product;
use crate::enums::RiskClass;
use crate::error::{FieldError, ProductError};
use crate::id::DatabaseIdGenerator;
use crate::table_modules::intern::models::product_model::ProductModel;
use crate::table_modules::ProductTM;

/// O nome de um produto.
#[nutype(sanitize(trim), validate(not_empty, len_char_max = 255))]
struct ProductName(String);

/// A densidade de um produto, em toneladas por unidade.
///
/// O `finite` vem antes do `greater`, e não é redundante: `f64::NAN > 0.0` é
/// **falso**, então um NaN passaria pela comparação sozinha — e contaminaria
/// todo cálculo de peso do contêiner adiante.
#[nutype(validate(finite, greater = 0.0))]
struct Density(f64);

/// Monta as regras de produto com o seu gerador de id.
///
/// O gerador chega injetado e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo.
pub(crate) fn product_tm<G>(
    id_generator: G,
) -> impl ProductTM + Send + Sync + Clone + use<G> + 'static
where
    G: DatabaseIdGenerator + Send + Sync + Clone + 'static,
{
    ProductTMImpl { id_generator }
}

/// A implementação, genérica sobre o gerador de id.
#[derive(Clone)]
struct ProductTMImpl<G> {
    /// De onde sai a identidade de um produto novo.
    id_generator: G,
}

impl<G: DatabaseIdGenerator> ProductTMImpl<G> {
    /// Examina nome e densidade, acumulando o que estiver errado.
    ///
    /// Os dois `try_new` acontecem antes de qualquer retorno: quem errou os dois
    /// campos recebe os dois de volta.
    fn checked(name: String, density: f64) -> Result<(ProductName, Density), ProductError> {
        let checked_name = ProductName::try_new(name);
        let checked_density = Density::try_new(density);

        let mut errors = Vec::new();
        if let Err(error) = &checked_name {
            errors.push(name_refused(error));
        }
        if checked_density.is_err() {
            errors.push(FieldError::new(
                "density",
                "Density must be greater than zero.",
            ));
        }

        let (Ok(name), Ok(density)) = (checked_name, checked_density) else {
            return Err(ProductError::Validation(errors));
        };

        Ok((name, density))
    }
}

impl<G: DatabaseIdGenerator> ProductTM for ProductTMImpl<G> {
    fn create(
        &self,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError> {
        let (name, density) = Self::checked(name, density)?;

        Ok(Box::new(ProductModel::new(
            self.id_generator.next(),
            name.into_inner(),
            density.into_inner(),
            risk_class,
        )))
    }

    fn update(
        &self,
        product: &dyn Product,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError> {
        let (name, density) = Self::checked(name, density)?;

        let mut model = ProductModel::from_domain(product);
        model.set_details(name.into_inner(), density.into_inner(), risk_class);
        Ok(Box::new(model))
    }
}

/// Comprimento máximo do nome, casando com a coluna `VARCHAR(255)`.
const MAX_NAME_LENGTH: usize = 255;

/// Traduz a recusa do nome na mensagem que o cliente lê.
fn name_refused(error: &ProductNameError) -> FieldError {
    match *error {
        ProductNameError::NotEmptyViolated => FieldError::new("name", "Name is required."),
        ProductNameError::LenCharMaxViolated => FieldError::new(
            "name",
            format!("Name must not exceed {MAX_NAME_LENGTH} characters."),
        ),
    }
}

#[cfg(test)]
#[path = "tests/product_tm_impl_test.rs"]
mod tests;
