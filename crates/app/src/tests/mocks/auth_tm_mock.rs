//! O mock de [`AuthTM`].

use mockall::mock;
use portmaster_domain::domain::User;
use portmaster_domain::error::AuthError;
use portmaster_domain::table_modules::AuthTM;

mock! {
    /// A regra de autenticação, sob controle do teste.
    pub(crate) AuthRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for AuthRules {
        fn clone(&self) -> Self;
    }

    impl AuthTM for AuthRules {
        fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError>;
    }
}
