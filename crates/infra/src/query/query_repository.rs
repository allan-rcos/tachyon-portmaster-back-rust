//! O contrato de quem executa uma consulta de leitura.

use crate::query::SqlDql;

/// Roda uma consulta e devolve a View que ela hidratou.
///
/// A única saída do lado de leitura. O que devolve objeto de domínio é
/// repositório de escrita, não isto.
#[trait_variant::make(Send)]
pub trait QueryRepository {
    /// Executa e devolve a View **por valor** — monomorfizada, sem `Box<dyn>`.
    ///
    /// Um resultado vazio é sucesso com View vazia, nunca um erro: decidir que
    /// "não achei nada" é problema é do chamador, não da execução.
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View>;
}
