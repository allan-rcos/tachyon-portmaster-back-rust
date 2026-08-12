//! O mock de [`RoleTM`].

use mockall::mock;
use portmaster_domain::domain::Role;
use portmaster_domain::error::RoleError;
use portmaster_domain::table_modules::RoleTM;

mock! {
    /// As regras de papel, sob controle do teste.
    pub(crate) RoleRules {}

    impl RoleTM for RoleRules {
        fn create(&self, name: String, permissions: Vec<String>)
            -> Result<Box<dyn Role>, RoleError>;

        fn update_permissions(&self, role: &dyn Role, permissions: Vec<String>)
            -> Result<Box<dyn Role>, RoleError>;
    }
}
