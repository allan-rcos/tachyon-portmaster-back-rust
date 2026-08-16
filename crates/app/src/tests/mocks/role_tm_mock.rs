//! O mock de [`RoleTM`].

use mockall::mock;
use portmaster_domain::domain::Role;
use portmaster_domain::error::RoleError;
use portmaster_domain::table_modules::RoleTM;

mock! {
    /// As regras de papel, sob controle do teste.
    pub(crate) RoleRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for RoleRules {
        fn clone(&self) -> Self;
    }

    impl RoleTM for RoleRules {
        fn create(&self, name: String, permissions: Vec<String>)
            -> Result<Box<dyn Role>, RoleError>;

        fn update_permissions(&self, role: &dyn Role, permissions: Vec<String>)
            -> Result<Box<dyn Role>, RoleError>;
    }
}
