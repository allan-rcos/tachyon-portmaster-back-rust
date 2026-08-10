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

/// A implementação, genérica sobre o gerador de id.
#[derive(Clone)]
pub(crate) struct ProductTMImpl<G> {
    /// De onde sai a identidade de um produto novo.
    id_generator: G,
}

impl<G: DatabaseIdGenerator> ProductTMImpl<G> {
    /// Monta o `TableModule` com o seu gerador de id.
    pub(crate) const fn new(id_generator: G) -> Self {
        Self { id_generator }
    }

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
mod tests {
    use super::*;
    use crate::table_modules::intern::helpers::fields_of::fields_of;
    use pretty_assertions::assert_eq;

    struct FixedIdGenerator;
    impl DatabaseIdGenerator for FixedIdGenerator {
        fn next(&self) -> String {
            "P1".to_string()
        }
    }

    fn table_module() -> ProductTMImpl<FixedIdGenerator> {
        ProductTMImpl::new(FixedIdGenerator)
    }

    #[test]
    fn cria_produto_valido() {
        let product = table_module()
            .create("Soja".into(), 0.75, RiskClass::None)
            .expect("os dados são válidos");

        assert_eq!(product.id(), "P1");
        assert_eq!(product.name(), "Soja");
        assert_eq!(product.density(), 0.75);
        assert_eq!(product.risk_class(), RiskClass::None);
    }

    #[test]
    fn recusa_densidade_que_nao_converte_quantidade_em_peso() {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let error = table_module()
                .create("Soja".into(), bad, RiskClass::None)
                .err()
                .unwrap_or_else(|| panic!("densidade {bad} deveria ser recusada"));

            let ProductError::Validation(fields) = error;
            assert_eq!(fields_of(&fields), vec!["density"], "densidade: {bad}");
        }
    }

    #[test]
    fn acumula_nome_e_densidade_invalidos() {
        let error = table_module()
            .create(String::new(), 0.0, RiskClass::None)
            .err()
            .expect("os dois campos são inválidos");

        let ProductError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["name", "density"]);
    }

    #[test]
    fn update_nao_altera_o_produto_recebido() {
        let original = table_module()
            .create("Soja".into(), 0.75, RiskClass::None)
            .expect("os dados são válidos");

        let updated = table_module()
            .update(
                original.as_ref(),
                "Soja tipo 2".into(),
                0.8,
                RiskClass::Class9Miscellaneous,
            )
            .expect("os dados são válidos");

        assert_eq!(original.name(), "Soja");
        assert_eq!(original.density(), 0.75);
        assert_eq!(updated.name(), "Soja tipo 2");
        assert_eq!(updated.id(), original.id());
    }
}
