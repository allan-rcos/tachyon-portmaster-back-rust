//! As impls de geração de id. Nenhuma sai do crate.

pub(crate) mod nano_id_generator;
pub(crate) mod xid_generator;

#[cfg(feature = "id-snowflake")]
pub(crate) mod snowflake_id_generator;
