//! Os dois geradores de id que **não** são identidade de entidade.
//!
//! Identidade de entidade nasce no `domain`, no `TableModule`. Estes dois são da
//! borda: o refresh token precisa ser opaco e imprevisível, o `request_id`
//! precisa ordenar no tempo para agrupar log. Nenhum dos dois é regra de
//! negócio, e por isso moram aqui.

pub mod random_id_generator;
pub mod sortable_id_generator;

pub(crate) mod interno;

pub use random_id_generator::RandomIdGenerator;
pub use sortable_id_generator::SortableIdGenerator;
