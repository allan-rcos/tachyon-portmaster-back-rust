//! # portmaster-domain
//!
//! O núcleo. Só regras de negócio: não depende de nenhum outro crate e não toca
//! banco, rede ou I/O.
//!
//! Exporta **apenas traits** — os traits de objeto de domínio (read-only), os
//! `TableModules`, a trait [`DomainProvider`] e [`register()`]. Models,
//! implementações de `TableModule` e helpers internos (gerador de id, hashers) são
//! privados ao crate e servidos pelos factories do provider.
//!
//! Essa reserva não é cerimônia. Se o `app` alcançasse o hasher de senha, ele
//! conseguiria gravar um usuário direto na `infra`, pulando a validação do
//! `TableModule` — e no dia em que a regra de senha mudasse, metade do sistema
//! estaria validando pela regra antiga.
//!
//! ## Ids
//!
//! Saem daqui como `String` base62. O `TableModule` gera o Snowflake `i64` e o
//! compacta antes de expor, porque escolher a identidade de uma entidade é regra
//! de negócio — não é o repositório que decide quem ela é. Só a `infra` volta a
//! ver o inteiro, ao tocar o `BIGINT`.
//!
//! ## Erros
//!
//! Tipados, nunca `anyhow` e nunca status HTTP: o domínio não sabe que existe
//! HTTP. E a validação **acumula** — um validator examina todos os campos e
//! devolve a lista inteira, para que o cliente conserte tudo de uma vez.

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

pub mod config;
pub mod enums;
pub mod error;
pub mod id;
pub mod models;
pub mod provider;
pub mod register;
pub mod security;
pub mod table_modules;

pub(crate) mod interno;

pub use config::DomainSecrets;
pub use provider::DomainProvider;
pub use register::register;
