//! Os controllers: wire ↔ Command.
//!
//! Um controller é a junção dos handlers de um recurso; um handler é o método.
//!
//! Fininhos de propósito. Um handler faz quatro coisas e nada além: confere se
//! há sessão, monta o Command, delega ao caso de uso e mapeia o resultado de
//! volta ao VO. Não há regra de negócio, orquestração nem transação aqui — se
//! aparecer, está no lugar errado.
//!
//! ## O 401 é o único status que nasce aqui
//!
//! Falta de sessão é a única coisa que o `app` não tem como saber, porque só
//! esta camada lê o token. Permissão (403), validação (422), ausência (404) e
//! conflito (409) vêm todos do `app`, traduzidos em `ports::error`.
//!
//! ## Cada recurso são três arquivos
//!
//! O **trait** declara os handlers em termos de VOs — sem axum, sem status, sem
//! negociação. A **impl** em `intern` é genérica sobre os casos de uso que o
//! `AppProvider` entrega, cujos tipos são innomeáveis. E o módulo de **rotas**
//! liga os dois ao axum, guardando ali todo o encanamento de extractor que o
//! router de cima não precisa ver.
//!
//! É a trait que torna os handlers chamáveis de um teste sem subir servidor, e é
//! ela que o [`ApiProvider`](crate::bootstrap::provider::ApiProvider) devolve — por RPITIT,
//! como todo o resto do grafo.

pub(crate) mod params;

pub(crate) mod account_controller;
pub(crate) mod account_routes;
pub(crate) mod auth_controller;
pub(crate) mod auth_routes;
pub(crate) mod container_controller;
pub(crate) mod container_routes;
pub(crate) mod manifest_controller;
pub(crate) mod manifest_routes;
pub(crate) mod metadata_controller;
pub(crate) mod metadata_routes;
pub(crate) mod metrics_controller;
pub(crate) mod metrics_routes;
pub(crate) mod product_controller;
pub(crate) mod product_routes;
pub(crate) mod role_controller;
pub(crate) mod role_routes;
pub(crate) mod server_controller;
pub(crate) mod server_routes;
pub(crate) mod user_controller;
pub(crate) mod user_routes;

pub(crate) mod intern;
