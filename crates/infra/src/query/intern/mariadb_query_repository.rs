//! A execução de consultas sobre o `MariaDB`.

use anyhow::Context;
use mysql_async::prelude::Queryable as _;
use mysql_async::Row;

use crate::query::{QueryRepository, SqlDql};
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// Monta o executor de consultas sobre o `MariaDB`.
///
/// O que ele guarda é o handle do banco; a transação em si vem do escopo da
/// tarefa, o que permite ao provider reconstruí-lo a cada chamada por custo
/// nenhum.
pub(crate) fn mariadb_query_repository<T>(
    transactions: T,
) -> impl QueryRepository + Sync + Clone + use<T> + 'static
where
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    MariadbQueryRepository { transactions }
}

/// A implementação sobre `MariaDB`.
#[derive(Clone)]
struct MariadbQueryRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T: MySqlTransaction + Send + Sync> QueryRepository for MariadbQueryRepository<T> {
    /// Executa o DQL e entrega as linhas para ele hidratar.
    ///
    /// O texto e os valores chegam separados do [`build`](SqlDql::build) e assim
    /// seguem até o servidor: a consulta é preparada, e os valores viajam como
    /// parâmetros. Não há ponto neste caminho onde um valor pudesse ser
    /// interpolado no texto.
    ///
    /// O `MariaDB` compara número com texto descartando o índice, então importa
    /// que cada valor chegue com o tipo que tem — e é quem monta a consulta quem
    /// o declara, não uma tradução aqui.
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View> {
        let (sql, params) = dql.build();
        let mut transaction = self.transactions.transaction().await?;

        let rows: Vec<Row> = transaction
            .exec(&sql, params)
            .await
            .with_context(|| format!("falha ao executar a consulta de leitura: {sql}"))?;

        dql.read(rows)
    }
}
