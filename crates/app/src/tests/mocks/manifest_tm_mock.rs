//! O mock de [`ManifestTM`].

#![allow(
    clippy::ref_option_ref,
    reason = "o predicado que o mock! gera recebe &Option<&T>; a assinatura é do macro, não deste arquivo"
)]

use mockall::mock;
use portmaster_domain::domain::{Container, ManifestCargo, ManifestChange, Product};
use portmaster_domain::error::ManifestError;
use portmaster_domain::table_modules::ManifestTM;

mock! {
    /// As regras do manifesto, sob controle do teste.
    pub(crate) ManifestRules {}

    impl ManifestTM for ManifestRules {
        fn load<'a>(&self, container: &dyn Container, product: &dyn Product, quantity: f64,
            current: Option<&'a dyn ManifestCargo>)
            -> Result<Box<dyn ManifestChange>, ManifestError>;
        fn unload<'a>(&self, container: &dyn Container, product: &dyn Product, quantity: f64,
            current: Option<&'a dyn ManifestCargo>)
            -> Result<Box<dyn ManifestChange>, ManifestError>;
    }
}
