//! O contrato do cache de leitura.

use std::sync::Arc;

/// Cache de leitura, indiferente ao que guarda.
#[trait_variant::make(Send)]
pub trait ReadCache {
    /// O valor sob a chave, se ainda válido.
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>>;

    /// Guarda o valor sob a chave, com o TTL desta camada.
    async fn put(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()>;

    /// Descarta uma chave.
    async fn invalidate(&self, key: &str) -> anyhow::Result<()>;

    /// Descarta tudo que começa com um prefixo.
    ///
    /// É o que uma escrita usa: alterar um produto invalida todas as listagens
    /// de produto de uma vez, sem que o caso de uso precise enumerar quais
    /// combinações de filtro existem por aí.
    async fn invalidate_prefix(&self, prefix: &str) -> anyhow::Result<()>;
}
