//! Regras de produto.

use crate::enums::RiskClass;
use crate::error::{ProductError, Validation};
use crate::id::IntIdGenerator;

use super::model::{Product, ProductModel};

/// Constrói e altera produtos.
pub trait ProductTM {
    /// Cria um produto novo, ainda não persistido.
    fn create(
        &self,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError>;

    /// Produz o produto com outros dados de catálogo.
    fn update(
        &self,
        product: &dyn Product,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError>;
}

/// Comprimento máximo do nome, casando com a coluna `VARCHAR(255)`.
const MAX_NAME_LENGTH: usize = 255;

/// A implementação, genérica sobre o gerador de id.
pub(crate) struct ProductTMImpl<G> {
    id_generator: G,
}

impl<G: IntIdGenerator> ProductTMImpl<G> {
    /// Monta o TableModule com o seu gerador de id.
    pub(crate) fn new(id_generator: G) -> Self {
        Self { id_generator }
    }

    /// Examina nome e densidade, acumulando o que estiver errado.
    fn validate(&self, name: &str, density: f64) -> Validation {
        let mut errors = Validation::new();

        if name.trim().is_empty() {
            errors.add("name", "Name is required.");
        } else if name.chars().count() > MAX_NAME_LENGTH {
            errors.add(
                "name",
                format!("Name must not exceed {MAX_NAME_LENGTH} characters."),
            );
        }

        // `is_finite` cobre NaN e infinito, que uma comparação sozinha deixaria
        // passar: `f64::NAN <= 0.0` é falso, e um NaN aqui contaminaria todo
        // cálculo de peso do contêiner adiante.
        errors.add_if(
            !density.is_finite() || density <= 0.0,
            "density",
            "Density must be greater than zero.",
        );

        errors
    }
}

impl<G: IntIdGenerator> ProductTM for ProductTMImpl<G> {
    fn create(
        &self,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError> {
        self.validate(&name, density)
            .into_result(())
            .map_err(ProductError::Validation)?;

        Ok(Box::new(ProductModel::new(
            self.id_generator.next(),
            name,
            density,
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
        self.validate(&name, density)
            .into_result(())
            .map_err(ProductError::Validation)?;

        let mut model = ProductModel::from_domain(product);
        model.set_details(name, density, risk_class);
        Ok(Box::new(model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::tm::fields_of;
    use pretty_assertions::assert_eq;

    struct FixedIdGenerator;
    impl IntIdGenerator for FixedIdGenerator {
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
                .expect("densidade {bad} deveria ser recusada");

            let ProductError::Validation(fields) = error;
            assert_eq!(fields_of(&fields), vec!["density"], "densidade: {bad}");
        }
    }

    #[test]
    fn acumula_nome_e_densidade_invalidos() {
        let error = table_module()
            .create("".into(), 0.0, RiskClass::None)
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
