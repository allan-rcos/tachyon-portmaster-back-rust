//! Papel: o pacote de permissões que se concede a um usuário.

pub(crate) mod model;
pub mod tm;

pub use model::Role;
pub use tm::RoleTM;
