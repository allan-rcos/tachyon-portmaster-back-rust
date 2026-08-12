//! O mock de [`ProductTM`].

use mockall::mock;
use portmaster_domain::domain::Product;
use portmaster_domain::enums::RiskClass;
use portmaster_domain::error::ProductError;
use portmaster_domain::table_modules::ProductTM;

mock! {
    /// As regras de produto, sob controle do teste.
    pub(crate) ProductRules {}

    impl ProductTM for ProductRules {
        fn create(&self, name: String, density: f64, risk_class: RiskClass)
            -> Result<Box<dyn Product>, ProductError>;
        fn update(&self, product: &dyn Product, name: String, density: f64,
            risk_class: RiskClass) -> Result<Box<dyn Product>, ProductError>;
    }
}
