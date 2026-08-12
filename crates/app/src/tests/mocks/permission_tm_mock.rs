//! O mock de [`PermissionTM`].

use mockall::mock;
use portmaster_domain::domain::Permission;
use portmaster_domain::error::MetadataError;
use portmaster_domain::table_modules::PermissionTM;

mock! {
    /// As regras de permissão, sob controle do teste.
    pub(crate) PermissionRules {}

    impl PermissionTM for PermissionRules {
        fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError>;
    }
}
