//! As impls de geração de id. Nenhuma sai do crate.

#[cfg(feature = "id-snowflake")]
pub(crate) mod snowflake_id_generator;
