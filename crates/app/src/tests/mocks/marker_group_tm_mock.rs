//! O mock de [`MarkerGroupTM`].

use mockall::mock;
use portmaster_domain::domain::MarkerGroup;
use portmaster_domain::error::MetadataError;
use portmaster_domain::table_modules::MarkerGroupTM;

mock! {
    /// As regras de grupo de marcador, sob controle do teste.
    pub(crate) MarkerGroupRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for MarkerGroupRules {
        fn clone(&self) -> Self;
    }

    impl MarkerGroupTM for MarkerGroupRules {
        fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError>;
    }
}
