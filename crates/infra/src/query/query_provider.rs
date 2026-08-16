//! Quem serve o executor do lado de leitura.

use crate::query::intern::mariadb_query_repository::mariadb_query_repository;
use crate::query::QueryRepository;
use crate::scope::ScopeProvider;

/// O executor de DQL.
///
/// Um só, e não um por consulta: o que varia é o DQL que chega por argumento,
/// e ele é uma função que o `app` importa direto. Não há factory de DQL neste
/// provider por isso mesmo — quem pede uma consulta não escolhe de um menu.
pub(crate) struct QueryProvider;

impl QueryProvider {
    /// Quem executa um DQL contra o banco.
    pub(crate) fn queries() -> anyhow::Result<impl QueryRepository + Sync + Clone + use<> + 'static>
    {
        Ok(mariadb_query_repository(ScopeProvider::database()?))
    }
}
