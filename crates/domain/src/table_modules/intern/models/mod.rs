//! As implementações dos objetos de domínio. Nenhuma sai do crate.
//!
//! Moram debaixo do `TableModule` porque é ele quem as constrói: um model é o
//! resultado de uma regra ter passado, e ninguém mais no sistema pode produzir
//! um. Guardá-los aqui faz a vizinhança dizer isso sozinha.

pub(crate) mod container_model;
pub(crate) mod manifest_cargo_model;
pub(crate) mod manifest_change_model;
pub(crate) mod marker_group_model;
pub(crate) mod marker_model;
pub(crate) mod permission_model;
pub(crate) mod product_model;
pub(crate) mod role_model;
pub(crate) mod user_model;
