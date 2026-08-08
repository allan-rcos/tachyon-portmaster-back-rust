//! O empréstimo da transação da requisição.
//!
//! É um `tokio::sync::OwnedMutexGuard`, e isso é deliberado: o guard **precisa**
//! atravessar o `.await` da query, porque não há como executar SQL na transação
//! sem segurá-la. Um mutex síncrono aqui seria o bug que o
//! `await_holding_lock` existe para pegar. Ver a nota em `.clippy.toml`.

use sqlx::mysql::MySql;
use sqlx::Transaction;

/// Acesso emprestado à transação corrente.
///
/// Enquanto vive, nenhuma outra parte da mesma requisição consegue a transação —
/// que é o correto, já que uma transação SQL é sequencial por natureza.
pub(crate) struct TransactionGuard {
    guard: tokio::sync::OwnedMutexGuard<Option<Transaction<'static, MySql>>>,
}

impl TransactionGuard {
    /// Embrulha o empréstimo já tirado do slot da requisição.
    ///
    /// Só a `MariadbUnitOfWork` chama: ela é quem garantiu, antes de chamar, que
    /// há transação no slot — e é essa garantia que sustenta o `expect` abaixo.
    pub(crate) const fn new(
        guard: tokio::sync::OwnedMutexGuard<Option<Transaction<'static, MySql>>>,
    ) -> Self {
        Self { guard }
    }

    /// A transação, para passar ao `sqlx`.
    #[allow(
        clippy::expect_used,
        reason = "invariante do tipo: `current()` só constrói o guard com a transação presente, e o guard impede que alguém a tire enquanto este empréstimo vive"
    )]
    pub(crate) fn as_mut(&mut self) -> &mut Transaction<'static, MySql> {
        self.guard
            .as_mut()
            .expect("o guard só é construído com transação presente")
    }
}
