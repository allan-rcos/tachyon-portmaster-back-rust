//! Os tetos de memória dos caches.
//!
//! Namespace: quatro números que só fazem sentido lidos juntos — o TTL e a
//! capacidade do cache de leitura, e quantos marcadores e metadados cabem.

/// Os limites dos caches em memória.
pub(crate) struct CacheLimits;

impl CacheLimits {
    /// Quanto tempo uma consulta cacheada continua válida.
    ///
    /// Curto de propósito. O cache aqui absorve rajadas de leitura repetida, não
    /// substitui o banco — e toda escrita invalida a chave que tocou, então a
    /// janela de dado velho é o intervalo entre duas leituras, não este TTL.
    pub(crate) const READ_CACHE_TTL_SECONDS: u64 = 30;
    /// Quantas consultas cacheadas cabem antes de o Moka começar a despejar.
    pub(crate) const READ_CACHE_CAPACITY: u64 = 10_000;
    /// Quantos marcadores cabem em memória.
    ///
    /// Um marcador é uma sessão de refresh viva; o teto existe para que uma enxurrada
    /// de logins não consuma memória sem limite.
    pub(crate) const MARKER_CACHE_CAPACITY: u64 = 100_000;
    /// Quantos metadados de sistema cabem em memória.
    ///
    /// Permissões e grupos são dezenas, registrados no boot e nunca mais alterados.
    pub(crate) const METADATA_CACHE_CAPACITY: u64 = 1_000;
}
