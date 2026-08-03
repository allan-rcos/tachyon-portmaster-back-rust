//! Contêiner: a unidade que se carrega, sela e despacha.

pub(crate) mod model;
pub mod tm;

pub use model::Container;
pub use tm::ContainerTM;
