//! # portmaster-domain
//!
//! O núcleo. Só regras de negócio: não depende de nenhum outro crate e não toca
//! banco, rede ou I/O.
//!
//! Exporta **apenas traits** — os traits de objeto de domínio (read-only), os
//! `TableModules`, os geradores de id de borda, a trait [`DomainProvider`] e
//! [`register()`]. Models, implementações de `TableModule` e helpers internos
//! (o gerador de identidade, os hashers) são privados ao crate e servidos pelos
//! factories do provider.
//!
//! Essa reserva não é cerimônia. Se o `app` alcançasse o hasher de senha, ele
//! conseguiria gravar um usuário direto na `infra`, pulando a validação do
//! `TableModule` — e no dia em que a regra de senha mudasse, metade do sistema
//! estaria validando pela regra antiga.
//!
//! ## Ids
//!
//! Os três geradores moram aqui, porque emitir identidade é contrato de negócio
//! — não é o repositório que decide quem uma entidade é. O id de banco sai como
//! `String` base62: o `TableModule` gera o Snowflake e o compacta antes de
//! expor, e só a `infra` volta a ver o inteiro, ao tocar o `BIGINT`. Ver
//! [`id`].
//!
//! ## Erros
//!
//! Tipados, nunca `anyhow` e nunca status HTTP: o domínio não sabe que existe
//! HTTP. E a validação **acumula** — um validator examina todos os campos e
//! devolve a lista inteira, para que o cliente conserte tudo de uma vez.

#![deny(unsafe_code)]
#![warn(missing_docs)]
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

pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod enums;
pub mod error;
pub mod id;
pub mod security;
pub mod table_modules;

pub use bootstrap::provider::DomainProvider;
pub use bootstrap::register::register;
pub use config::DomainSecrets;
