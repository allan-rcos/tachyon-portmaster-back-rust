//! A execução de consultas sobre o `MariaDB`.

use anyhow::Context;

use crate::database::interno::mariadb_unit_of_work::MariadbUnitOfWork;
use crate::query::sql::{Bind, SqlQuery};
use crate::query::{QueryRepository, SqlDql};

/// A implementação sobre `MariaDB`.
///
/// Sem estado: a transação vem do escopo da requisição, o que permite ao
/// provider reconstruí-la a cada chamada por custo nenhum.
#[derive(Clone)]
pub(crate) struct MariadbQueryRepository;

impl MariadbQueryRepository {
    /// Monta o repositório.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl QueryRepository for MariadbQueryRepository {
    /// Executa o DQL e entrega as linhas para ele hidratar.
    ///
    /// ## Por que `AssertSqlSafe` é legítimo aqui
    ///
    /// É o que o sqlx 0.9 exige de todo SQL montado em tempo de execução, e a
    /// exigência é justa: o texto não é mais uma constante que se lê no
    /// arquivo. A afirmação se sustenta porque o `Select` **nunca interpola
    /// valor nenhum** — tudo que vem de fora entra como `Bind`, e o único
    /// trecho de texto variável são placeholders `?` contados a partir do
    /// tamanho de um `Vec`.
    ///
    /// ## Por que os binds são ligados por tipo
    ///
    /// Um id enviado como string faz o `MariaDB` comparar número com texto e
    /// descartar o índice.
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View> {
        let SqlQuery { sql, binds } = dql.build();
        let mut transaction = MariadbUnitOfWork::current().await?;

        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.clone()));
        for bind in binds {
            query = match bind {
                Bind::Int(value) => query.bind(value),
                Bind::Text(value) => query.bind(value),
            };
        }

        let rows = query
            .fetch_all(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao executar a consulta de leitura: {sql}"))?;

        dql.read(rows)
    }
}
