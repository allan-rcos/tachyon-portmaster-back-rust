//! O cache de marcadores, com expiração por entrada.
//!
//! Vira um tipo em vez de um `type` alias mais uma função solta porque a
//! política de expiração e o construtor só fazem sentido juntos: montar o cache
//! sem o `MarkerExpiry` daria um mapa em que toda sessão de refresh vive para
//! sempre, e o bug seria invisível até alguém reparar que o logout não expira.

use std::sync::Arc;
use std::time::{Duration, Instant};

use moka::future::Cache;
use moka::policy::Expiry;

/// A chave de um marcador: o grupo e o digest do valor.
type MarkerKey = (String, String);

/// O valor de um marcador: a flag e quanto tempo ela ainda vale.
///
/// O prazo viaja junto do valor porque o Moka decide a expiração a partir da
/// entrada, não da chamada que a inseriu — e cada sessão vence no seu próprio
/// tempo, não num vencimento global.
type MarkerValue = (bool, Duration);

/// Ensina o Moka a ler o prazo que cada entrada carrega.
struct MarkerExpiry;

impl Expiry<MarkerKey, MarkerValue> for MarkerExpiry {
    fn expire_after_create(
        &self,
        _key: &MarkerKey,
        value: &MarkerValue,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(value.1)
    }

    fn expire_after_update(
        &self,
        _key: &MarkerKey,
        value: &MarkerValue,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        // Invalidar renova o prazo em vez de herdar o que restava do anterior:
        // a marca desligada precisa sobreviver ao menos tanto quanto o token que
        // ela recusa, senão expirar equivaleria a esquecer o logout.
        Some(value.1)
    }
}

/// Cache de marcadores, com expiração por entrada.
#[derive(Clone)]
pub(crate) struct MarkerCache(pub(crate) Arc<Cache<MarkerKey, MarkerValue>>);

impl MarkerCache {
    /// Monta o cache com a política de expiração por entrada já instalada.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(
            Cache::builder()
                .max_capacity(capacity)
                .expire_after(MarkerExpiry)
                .build(),
        ))
    }
}
