//! O que cada escrita torna obsoleto.
//!
//! Namespace: os cinco conjuntos existem para serem lidos lado a lado. Uma
//! escrita que derruba prefixos demais só custa uma releitura; uma que derruba
//! de menos serve dado velho, e é por isso que o painel aparece em quase todos.

use crate::cache::cache_key::CacheKey;

/// Os prefixos que cada tipo de escrita invalida.
pub(crate) struct Invalidation;

impl Invalidation {
    /// O que uma escrita de produto torna obsoleto.
    ///
    /// O painel entra junto porque conta `registered_products`: cadastrar um produto
    /// muda um número que nada em `product:` alcança.
    pub(crate) const PRODUCT_WRITE: &[&str] = &[CacheKey::PRODUCT, CacheKey::METRICS];

    /// O que uma escrita de contêiner torna obsoleto.
    pub(crate) const CONTAINER_WRITE: &[&str] = &[CacheKey::CONTAINER, CacheKey::METRICS];

    /// O que um embarque ou desembarque torna obsoleto.
    ///
    /// O contêiner muda de peso e de status, o resumo muda de carga e de telemetria,
    /// e o painel muda a carga do pátio — três leituras diferentes atingidas por uma
    /// operação só.
    pub(crate) const MANIFEST_WRITE: &[&str] = &[CacheKey::CONTAINER, CacheKey::METRICS];

    /// O que uma escrita de usuário torna obsoleto.
    ///
    /// `account:` porque a conta é a mesma pessoa vista de outro ângulo, e `role:`
    /// porque a listagem de papéis carrega `user_count`.
    pub(crate) const USER_WRITE: &[&str] = &[CacheKey::USER, CacheKey::ACCOUNT, CacheKey::ROLE];

    /// O que uma escrita de papel torna obsoleto.
    ///
    /// `user:` e `account:` porque os dois trazem os papéis aninhados: trocar as
    /// permissões de um papel muda toda conta que o carrega.
    pub(crate) const ROLE_WRITE: &[&str] = &[CacheKey::ROLE, CacheKey::USER, CacheKey::ACCOUNT];
}
