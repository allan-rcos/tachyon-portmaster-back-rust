//! O mock de [`ContainerTM`].

use mockall::mock;
use portmaster_domain::domain::Container;
use portmaster_domain::error::ContainerError;
use portmaster_domain::table_modules::ContainerTM;

mock! {
    /// As regras de contêiner, sob controle do teste.
    pub(crate) ContainerRules {}

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
