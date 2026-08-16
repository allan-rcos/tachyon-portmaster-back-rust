//! O mock de [`ContainerTM`].

use mockall::mock;
use portmaster_domain::domain::Container;
use portmaster_domain::error::ContainerError;
use portmaster_domain::table_modules::ContainerTM;

mock! {
    /// As regras de contêiner, sob controle do teste.
    pub(crate) ContainerRules {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for ContainerRules {
        fn clone(&self) -> Self;
    }

    impl ContainerTM for ContainerRules {
        fn create(&self, code: String, max_capacity: f64)
            -> Result<Box<dyn Container>, ContainerError>;
        fn update(&self, container: &dyn Container, max_capacity: f64)
            -> Result<Box<dyn Container>, ContainerError>;
        fn seal(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;
        fn dispatch(&self, container: &dyn Container)
            -> Result<Box<dyn Container>, ContainerError>;
    }
}
