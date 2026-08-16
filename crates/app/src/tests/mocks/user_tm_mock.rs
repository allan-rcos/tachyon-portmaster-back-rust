//! O mock de [`UserTM`].

use mockall::mock;
use portmaster_domain::domain::{Role, User};
use portmaster_domain::error::UserError;
use portmaster_domain::table_modules::UserTM;

mock! {
    /// As regras de usuário, sob controle do teste.
    pub(crate) UserRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for UserRules {
        fn clone(&self) -> Self;
    }

    impl UserTM for UserRules {
        fn create(&self, name: String, email: String, password: String,
            roles: Vec<Box<dyn Role>>) -> Result<Box<dyn User>, UserError>;
        fn update(&self, user: &dyn User, name: String, email: String)
            -> Result<Box<dyn User>, UserError>;
        fn change_password(&self, user: &dyn User, new_password: String)
            -> Result<Box<dyn User>, UserError>;
        fn update_roles(&self, user: &dyn User, roles: Vec<Box<dyn Role>>)
            -> Result<Box<dyn User>, UserError>;
    }
}
