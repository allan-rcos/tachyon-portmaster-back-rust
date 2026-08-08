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

pub mod account;
pub mod auth;
pub mod container;
pub mod manifest;
pub mod metadata;
pub mod metrics;
pub mod params;
pub mod product;
pub mod role;
pub mod server;
pub mod user;
