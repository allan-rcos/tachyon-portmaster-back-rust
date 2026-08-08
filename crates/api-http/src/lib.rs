//! # portmaster-api-http
//!
//! Apresentação HTTP. Depende só do `app`, e adapta o mundo REST para os
//! `UseCases`: mapeia wire → Command na entrada e resultado → wire na saída.
//!
//! Todo o HTTP mora aqui — router, middlewares, handlers, negociação de conteúdo
//! e JWT. Nenhuma outra camada conhece token, `FlatBuffers` ou status HTTP.
//!
//! O corpo é negociado por requisição entre `FlatBuffers` (binário, o formato
//! nativo) e JSON, com dois processos deliberadamente separados: as *strategies*
//! sabem serializar um formato e nada sobre o payload; as *factories* sabem os
//! dados de uma resposta e nada sobre o formato negociado.
//!
//! O `unsafe` é `deny` e não `forbid`: o código escrito à mão nesta camada não o
//! usa, mas o módulo de wire gerado pelo planus usa — é uma lib de serialização
//! zero-copy — e `forbid` não admite exceção local nenhuma.

#![deny(unsafe_code)]
// O relaxamento vale só no passe `cfg(test)` — ver o `reason` abaixo.
#![cfg_attr(
    test,
    allow(
        clippy::indexing_slicing,
        clippy::panic,
        clippy::float_cmp,
        clippy::unreachable,
        clippy::disallowed_types,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "asserção de teste: `panic!`, `v[0]` e float_cmp são a forma normal de escrever o teste, e falhar alto é o comportamento desejado, e um fake pode usar std::sync::Mutex"
    )
)]

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
