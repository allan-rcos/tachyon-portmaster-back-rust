//! A consulta compilada: o texto e os valores a ligar.

use crate::query::sql::Bind;

/// Uma consulta pronta para executar.
///
/// É o que um `SqlDql` produz e o que o
/// [`QueryRepository`](crate::query::query_repository::QueryRepository) consome. O repositório nunca monta
/// SQL: ele recebe isto e roda.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlQuery {
    /// O texto, com placeholders posicionais.
    pub(crate) sql: String,
    /// Os valores, na ordem dos placeholders.
    pub(crate) binds: Vec<Bind>,
}
