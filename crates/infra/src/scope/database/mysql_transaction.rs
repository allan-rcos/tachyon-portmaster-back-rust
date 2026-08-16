//! Como um repositório alcança a transação da tarefa.

use mysql_async::Transaction;
use tokio::sync::OwnedMappedMutexGuard;

/// O empréstimo da transação da tarefa.
///
/// É o guard do `tokio` sem embrulho nosso. `Owned` porque ele **precisa**
/// atravessar o `.await` da query — não há como executar SQL na transação sem
/// segurá-la; ver a exceção do `await_holding_lock` em `.clippy.toml`. E
/// *mapeado* para a transação em si porque o `Option` do slot é assunto de quem
/// abre: quem recebe este empréstimo já tem transação, e por isso não há
/// `expect` nenhum no caminho.
///
/// `pub(super)` para não competir com o export do arquivo — quem o recebe nunca
/// precisa nomeá-lo.
pub(super) type TransactionGuard =
    OwnedMappedMutexGuard<Option<Transaction<'static>>, Transaction<'static>>;

/// A transação da tarefa, para quem executa SQL.
///
/// `pub(crate)`: é o que mantém a conexão invisível para fora da `infra`.
#[trait_variant::make(Send)]
pub(crate) trait MySqlTransaction {
    /// A transação da tarefa, **abrindo-a** se ainda não houver.
    ///
    /// Abrir aqui, e não num `begin` que alguém precise lembrar de chamar, é o
    /// que faz um caso de uso que só leu da memória nunca abrir transação
    /// nenhuma. Fora de um escopo isto falha, como o `getTransaction()` do PHP:
    /// abrir uma transação que ninguém conseguiria consultar depois seria pior.
    async fn transaction(&self) -> anyhow::Result<TransactionGuard>;
}
