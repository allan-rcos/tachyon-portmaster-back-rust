//! O mock de [`ProductTM`].

use mockall::mock;
use portmaster_domain::domain::Product;
use portmaster_domain::enums::RiskClass;
use portmaster_domain::error::ProductError;
use portmaster_domain::table_modules::ProductTM;

mock! {
    /// As regras de produto, sob controle do teste.
    pub(crate) ProductRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for ProductRules {
        fn clone(&self) -> Self;
    }

    impl ProductTM for ProductRules {
        fn create(&self, name: String, density: f64, risk_class: RiskClass)
            -> Result<Box<dyn Product>, ProductError>;
        fn update(&self, product: &dyn Product, name: String, density: f64,
            risk_class: RiskClass) -> Result<Box<dyn Product>, ProductError>;
    }
}
