//! Manifesto: a carga de um contêiner, e o efeito de movimentá-la.

pub(crate) mod model;
pub mod tm;

pub use model::{ManifestCargo, ManifestChange};
pub use tm::ManifestTM;
