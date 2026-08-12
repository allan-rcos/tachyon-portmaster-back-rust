//! O mock de [`MarkerTM`].

use mockall::mock;
use portmaster_domain::domain::Marker;
use portmaster_domain::error::MarkerError;
use portmaster_domain::table_modules::MarkerTM;

mock! {
    /// As regras de marcador, sob controle do teste.
    pub(crate) MarkerRules {}

    impl MarkerTM for MarkerRules {
        fn create(&self, group: String, plain: &str, flag: bool)
            -> Result<Box<dyn Marker>, MarkerError>;
    }
}
