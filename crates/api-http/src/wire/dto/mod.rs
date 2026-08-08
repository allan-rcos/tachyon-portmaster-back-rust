//! Os DTOs do wire, e a factory de cada um, lado a lado.
//!
//! Um DTO é a mensagem como a apresentação a vê; a factory ao lado sabe lê-la
//! dos dois formatos (requisição) ou escrevê-la na tabela do planus (resposta).
//! Ficam no mesmo diretório por feature porque é assim que se lê: quem mexe numa
//! mensagem mexe nos dois arquivos.

pub(crate) mod account;
pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod container;
pub(crate) mod manifest;
pub(crate) mod metadata;
pub(crate) mod metrics;
pub(crate) mod product;
pub(crate) mod server;
