//! O mapa do Moka, com expiração por entrada.
//!
//! Vira um tipo em vez de um `type` alias mais uma função solta porque a
//! política de expiração e o construtor só fazem sentido juntos: montar o mapa
//! sem o [`EntryExpiry`] daria um cache em que toda sessão de refresh vive para
//! sempre, e o bug seria invisível até alguém reparar que o logout não expira.

use std::sync::Arc;
use std::time::{Duration, Instant};

use moka::future::Cache;
use moka::policy::Expiry;

/// A chave de uma entrada: o grupo e a identificação dentro dele.
pub(super) type EntryKey = (String, String);

/// O valor de uma entrada: os bytes e quanto tempo ela ainda vale.
///
/// O prazo viaja junto do valor porque o Moka decide a expiração a partir da
/// entrada, não da chamada que a inseriu. `None` é o que nunca vence.
pub(super) type EntryValue = (Arc<Vec<u8>>, Option<Duration>);

/// Ensina o Moka a ler o prazo que cada entrada carrega.
struct EntryExpiry;

impl Expiry<EntryKey, EntryValue> for EntryExpiry {
    fn expire_after_create(
        &self,
        _key: &EntryKey,
        value: &EntryValue,
        _created_at: Instant,
    ) -> Option<Duration> {
        value.1
    }

    /// Renova o prazo a cada escrita, em vez de herdar o restante do anterior.
    ///
    /// A marca desligada precisa sobreviver ao menos tanto quanto o token que
    /// ela recusa: se ela expirasse antes, o token voltaria a ser aceito — e
    /// expirar equivaleria a esquecer o logout.
    fn expire_after_update(
        &self,
        _key: &EntryKey,
        value: &EntryValue,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        value.1
    }
}

/// Um mapa em memória com política própria.
#[derive(Clone)]
pub(crate) struct MokaCache(pub(super) Arc<Cache<EntryKey, EntryValue>>);

impl MokaCache {
    /// Monta o mapa com o teto dado e a expiração por entrada instalada.
    ///
    /// `support_invalidation_closures` é o que permite descartar um grupo
    /// inteiro sem conhecer as chaves dele.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(
            Cache::builder()
                .max_capacity(capacity)
                .expire_after(EntryExpiry)
                .support_invalidation_closures()
                .build(),
        ))
    }
}
