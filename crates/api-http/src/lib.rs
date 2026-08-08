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

// `deny` e não `forbid`: o código escrito à mão nesta camada não usa `unsafe`,
// mas o módulo de wire gerado pelo planus usa — é uma lib de serialização
// zero-copy — e `forbid` não admite exceção local nenhuma.
#![deny(unsafe_code)]
// A blindagem de pânico do grupo C (tmp/clippy.md) vale para código de
// produção. Numa asserção de teste, `panic!`, `v[0]` e `assert_eq!` sobre float
// são a forma normal de escrever o teste, e não um risco: se o índice estourar
// ou o float divergir, o teste falha — que é exatamente o que se quer.
//
// O relaxamento é só do passe `cfg(test)`. O passe de biblioteca continua
// cobrindo o código de produção com os lints em `deny`, e `--all-targets` roda
// os dois.
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
        reason = "asserção de teste: falhar alto é o comportamento desejado, e um fake pode usar std::sync::Mutex"
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
