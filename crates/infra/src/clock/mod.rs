//! A hora corrente, injetada.
//!
//! Um dos dois recursos que o `.clippy.toml` proíbe de pegar do ar — o outro
//! são os ids. Quem precisa da hora recebe um [`Clock`] do provider.

#[allow(
    clippy::module_inception,
    reason = "o módulo `clock` exporta o tipo `Clock`: nome do arquivo = nome do tipo"
)]
pub mod clock;

pub(crate) mod interno;

pub use clock::Clock;
