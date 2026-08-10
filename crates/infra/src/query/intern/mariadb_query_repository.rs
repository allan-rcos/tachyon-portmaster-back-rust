//! A execução de consultas sobre o `MariaDB`.

use anyhow::Context;

use crate::query::{QueryRepository, SqlDql};
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// A implementação sobre `MariaDB`.
///
/// O que ele guarda é o handle do banco; a transação em si vem do escopo da
/// tarefa, o que permite ao provider reconstruí-lo a cada chamada por custo
/// nenhum.
#[derive(Clone)]
pub(crate) struct MariadbQueryRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T> MariadbQueryRepository<T> {
    /// Monta o repositório.
    pub(crate) const fn new(transactions: T) -> Self {
        Self { transactions }
    }
}

impl<T: MySqlTransaction + Send + Sync> QueryRepository for MariadbQueryRepository<T> {
    /// Executa o DQL e entrega as linhas para ele hidratar.
    ///
    /// Não há `AssertSqlSafe` no caminho, e é por construção: o `QueryBuilder`
    /// do sqlx separa o texto do valor no próprio tipo, então não existe ponto
    /// onde um valor pudesse ser interpolado no SQL. Antes essa garantia era uma
    /// afirmação nossa sobre um construtor nosso, e valia enquanto ninguém
    /// escrevesse a linha errada dentro dele.
    ///
    /// O `MariaDB` compara número com texto descartando o índice, então os
    /// valores continuam ligados pelo tipo que têm — o que agora é o
    /// `push_bind` de quem monta a consulta, e não uma tradução aqui.
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View> {
        let mut builder = dql.build();
        let sql = builder.sql().as_str().to_owned();
        let mut transaction = self.transactions.transaction().await?;

        let rows = builder
            .build()
            .fetch_all(&mut **transaction)
            .await
            .with_context(|| format!("falha ao executar a consulta de leitura: {sql}"))?;

        dql.read(rows)
    }
}
