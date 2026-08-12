//! O contrato do cache de leitura.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Guarda o resultado de uma consulta de leitura.
///
/// Um repositório só para todas as `View`, e não um por recurso: o que ele
/// guarda é sempre a mesma coisa — o que uma consulta devolveu.
///
/// A **chave** vem de [`Dql::cache_key`](crate::query::Dql::cache_key), e é o
/// DQL quem a escreve porque é ele quem sabe o que a consulta é. Antes cada
/// serviço a montava à mão, com um prefixo próprio e a lembrança de incluir cada
/// filtro.
///
/// O **grupo** é o recorte que uma escrita derruba. Quem chama o escolhe, e é
/// por isso que uma escrita de produto não precisa saber quais listagens
/// existem: ela descarta o grupo `product` inteiro.
#[trait_variant::make(Send)]
pub trait ViewCacheRepository {
    /// A View guardada sob esta chave, se houver.
    ///
    /// Uma entrada que não desserializa é tratada como ausência: ela é de um
    /// formato anterior, e recalcular em silêncio evita que um deploy vire
    /// incidente.
    async fn get<V: DeserializeOwned + 'static>(
        &self,
        group: &str,
        key: &str,
    ) -> anyhow::Result<Option<V>>;

    /// Guarda a View sob esta chave.
    async fn put<V: Serialize + Sync + 'static>(
        &self,
        group: &str,
        key: &str,
        view: &V,
    ) -> anyhow::Result<()>;

    /// Descarta o grupo inteiro.
    async fn invalidate(&self, group: &str) -> anyhow::Result<()>;
}
