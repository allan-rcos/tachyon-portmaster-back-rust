//! O contrato de persistência de marker.

use portmaster_domain::domain::Marker;

/// Marcadores booleanos com prazo.
///
/// A `infra` nunca sabe o que uma marca significa — só que é um booleano com
/// validade, num grupo conhecido.
#[trait_variant::make(Send)]
pub trait MarkerRepository {
    /// Grava a marca, aplicando as regras de transição.
    async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()>;

    /// Se a marca está válida.
    ///
    /// Marca inexistente, expirada ou desligada respondem igual: `false`. Quem
    /// pergunta quer saber se pode seguir, não por que não pode.
    async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool>;
}
