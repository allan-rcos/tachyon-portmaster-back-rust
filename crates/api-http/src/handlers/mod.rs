//! Os handlers: wire ↔ Command.
//!
//! Fininhos de propósito. Um handler faz quatro coisas e nada além: confere se
//! há sessão, monta o Command, delega ao caso de uso e mapeia o resultado de
//! volta ao wire. Não há regra de negócio, orquestração nem transação aqui — se
//! aparecer, está no lugar errado.
//!
//! ## O 401 é o único status que nasce aqui
//!
//! Falta de sessão é a única coisa que o `app` não tem como saber, porque só
//! esta camada lê o token. Permissão (403), validação (422), ausência (404) e
//! conflito (409) vêm todos do `app`, traduzidos em [`crate::error`].
//!
//! ## Por que cada recurso é uma struct genérica
//!
//! Os casos de uso que o `AppProvider` entrega têm tipos **innomeáveis** — só
//! existem depois da monomorfização. Um handler não consegue declarar o tipo do
//! que recebe, então recebe por generic. É a mesma razão que faz o router ser
//! genérico sobre o provider.

pub(crate) mod account;
pub(crate) mod auth;
pub(crate) mod container;
pub(crate) mod manifest;
pub(crate) mod metadata;
pub(crate) mod metrics;
pub(crate) mod product;
pub(crate) mod role;
pub(crate) mod server;
pub(crate) mod user;

use serde::Deserialize;

/// Os filtros de uma listagem paginada por cursor.
///
/// Todos opcionais: uma listagem sem querystring é a primeira página com os
/// padrões da `infra`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct PageParams {
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
    /// Termo de busca.
    pub(crate) search: Option<String>,
}

/// Os filtros da listagem de contêineres.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ContainerPageParams {
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
    /// Termo de busca sobre o código.
    pub(crate) search: Option<String>,
    /// Restringe a um status, pelo nome do enum.
    pub(crate) status: Option<String>,
    /// Restringe a um conjunto de status, separados por vírgula.
    pub(crate) status_in: Option<String>,
}

/// Os filtros da listagem de resumos.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct SummaryPageParams {
    /// Restringe a um contêiner.
    pub(crate) id: Option<String>,
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
}

/// Os filtros da listagem de usuários, que pagina por página e não por cursor.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct UserPageParams {
    /// Página, começando em 1.
    pub(crate) page: Option<u32>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
}

/// O filtro da listagem de permissões.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct SearchParams {
    /// Trecho do slug.
    pub(crate) search: Option<String>,
}
