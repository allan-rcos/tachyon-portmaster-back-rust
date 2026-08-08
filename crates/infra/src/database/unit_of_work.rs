//! O contrato da transação por requisição.

/// Gerencia o ciclo da transação — e nada mais.
///
/// Não executa consulta: o que ela oferece é começo, fim e desistência. Manter a
/// execução fora daqui é o que impede o `app` de rodar SQL.
#[trait_variant::make(Send)]
pub trait UnitOfWork {
    /// Abre a transação.
    async fn begin(&self) -> anyhow::Result<()>;

    /// Confirma o que foi feito.
    async fn commit(&self) -> anyhow::Result<()>;

    /// Descarta o que foi feito.
    ///
    /// Idempotente: chamar sem transação aberta não é erro, porque o caminho de
    /// falha frequentemente não sabe se chegou a abrir uma.
    async fn rollback(&self) -> anyhow::Result<()>;
}
