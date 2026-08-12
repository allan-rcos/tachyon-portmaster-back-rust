//! O mock de [`AuthTM`].

use mockall::mock;
use portmaster_domain::domain::User;
use portmaster_domain::error::AuthError;
use portmaster_domain::table_modules::AuthTM;

mock! {
    /// A regra de autenticação, sob controle do teste.
    pub(crate) AuthRules {}

    impl AuthTM for AuthRules {
        fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError>;
    }
}
