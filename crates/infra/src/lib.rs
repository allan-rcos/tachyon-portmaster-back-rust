//! # portmaster-infra
//!
//! Adaptadores de I/O. Depende só do `domain`, e só para pegar os traits de
//! objeto de domínio e implementá-los nos seus entities.
//!
//! É dona dos **ports de I/O** — repositórios, `UnitOfWork`, cache, `Logger`.
//! Eles ficam aqui e não no `domain` porque quem os consome é o `app`, que
//! conhece esta camada; o `domain` não sabe que existe banco.
//!
//! ## O `i64` para de existir aqui
//!
//! Esta é a única camada que enxerga o inteiro do Snowflake. Ao ler uma linha
//! deriva o base62 que o trait expõe; ao gravar, decodifica de volta para
//! `BIGINT`. Se o id físico mudar um dia, só este mapeamento muda.
//!
//! ## O que não mora aqui
//!
//! **JWT.** É assunto exclusivo do `api-http`: nem esta camada nem o `app` sabem
//! o que é um token. O que existe aqui é a primitiva de marcador — um booleano
//! com prazo — e é a apresentação que decide chamar aquilo de sessão.

#![deny(unsafe_code)]
#![warn(missing_docs)]
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

pub mod cache;
pub mod config;
pub mod database;
pub mod id;
pub mod logging;
pub mod provider;
pub mod query;
pub mod register;
pub mod repository;

pub(crate) mod entity;
pub(crate) mod interno;
pub(crate) mod text;

pub use provider::InfraProvider;
pub use register::register;
