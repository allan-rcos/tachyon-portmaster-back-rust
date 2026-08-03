//! # portmaster-api-http
//!
//! Apresentação HTTP. Depende só do `app`, e adapta o mundo REST para os
//! UseCases: mapeia wire → Command na entrada e resultado → wire na saída.
//!
//! Todo o HTTP mora aqui — router, middlewares, handlers, negociação de conteúdo
//! e JWT. Nenhuma outra camada conhece token, FlatBuffers ou status HTTP.
//!
//! O corpo é negociado por requisição entre FlatBuffers (binário, o formato
//! nativo) e JSON, com dois processos deliberadamente separados: as *strategies*
//! sabem serializar um formato e nada sobre o payload; as *factories* sabem os
//! dados de uma resposta e nada sobre o formato negociado.

// `deny` e não `forbid`: o código escrito à mão nesta camada não usa `unsafe`,
// mas o módulo de wire gerado pelo planus usa — é uma lib de serialização
// zero-copy — e `forbid` não admite exceção local nenhuma.
#![deny(unsafe_code)]

pub(crate) mod wire;

pub mod config;

pub(crate) mod error;

pub(crate) mod token;

pub(crate) mod cookie;
pub(crate) mod session;

pub(crate) mod middleware;

pub(crate) mod handlers;

mod router;

pub use router::router;
