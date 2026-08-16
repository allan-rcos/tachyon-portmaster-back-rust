//! O mock de [`PermissionTM`].

use mockall::mock;
use portmaster_domain::domain::Permission;
use portmaster_domain::error::MetadataError;
use portmaster_domain::table_modules::PermissionTM;

mock! {
    /// As regras de permissão, sob controle do teste.
    pub(crate) PermissionRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for PermissionRules {
        fn clone(&self) -> Self;
    }

    impl PermissionTM for PermissionRules {
        fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError>;
    }
}
