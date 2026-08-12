//! O mock de [`MarkerGroupTM`].

use mockall::mock;
use portmaster_domain::domain::MarkerGroup;
use portmaster_domain::error::MetadataError;
use portmaster_domain::table_modules::MarkerGroupTM;

mock! {
    /// As regras de grupo de marcador, sob controle do teste.
    pub(crate) MarkerGroupRules {}

    impl MarkerGroupTM for MarkerGroupRules {
        fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError>;
    }
}
