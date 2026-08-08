//! O tamanho do pool de conexões.
//!
//! Não é runtime: é decisão de arquitetura, escolhida por feature de
//! compilação. O valor certo depende de quantas conexões o `MariaDB` do outro
//! lado aguenta, que também não muda por deploy.

/// Conexões máximas no pool.
///
/// Não é runtime: é decisão de arquitetura, e o valor certo depende de quantas
/// conexões o `MariaDB` do outro lado aguenta — que também não muda por deploy.
#[cfg(feature = "pool-default")]
pub(crate) const POOL_MAX_CONNECTIONS: u32 = 32;
/// Conexões máximas no pool, para instalações grandes.
#[cfg(all(feature = "pool-large", not(feature = "pool-default")))]
pub(crate) const POOL_MAX_CONNECTIONS: u32 = 128;
