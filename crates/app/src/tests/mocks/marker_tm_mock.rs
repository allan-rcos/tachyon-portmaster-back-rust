//! O mock de [`MarkerTM`].

use mockall::mock;
use portmaster_domain::domain::Marker;
use portmaster_domain::error::MarkerError;
use portmaster_domain::table_modules::MarkerTM;

mock! {
    /// As regras de marcador, sob controle do teste.
    pub(crate) MarkerRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for MarkerRules {
        fn clone(&self) -> Self;
    }

    impl MarkerTM for MarkerRules {
        fn create(&self, group: String, plain: &str, flag: bool)
            -> Result<Box<dyn Marker>, MarkerError>;
    }
}
