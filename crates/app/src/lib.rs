//! # portmaster-app
//!
//! Orquestração. Depende de `domain` (`TableModules`, traits de objeto) e de
//! `infra` (repositories, escopo da tarefa, cache).
//!
//! Dono dos **Command DTOs**, dos **Query DTOs** e dos traits de `Service` — que
//! são exatamente os ports de que a apresentação precisa, e por isso são
//! definidos aqui: o `api-http` conhece só esta camada.
//!
//! Um **service** é o agrupamento; um **caso de uso** é o método dele, como um
//! handler é o método do controller. `RoleService::create` é um caso de uso.
//!
//! É aqui que a autorização acontece. Cada caso de uso protegido declara no
//! construtor a permissão que exige e a confere na primeira linha, contra o
//! `UserContext` que veio no Command — o contexto chega por argumento, nunca de
//! um estado global. É também o único lugar que abre o escopo da tarefa: o
//! `MasterScope::run` marca onde a unidade de trabalho começa e termina, e a
//! camada não sabe o que a `infra` carrega dentro dele.
//!
//! ## O que esta camada reexporta, e por quê
//!
//! O `api-http` conhece só o `app`. Então tudo que ele precisa e que nasce
//! abaixo — o trait `User` para mapear ao fio, as Views de leitura, o `Logger`,
//! os geradores de id — sai daqui. Não é conveniência: é o que mantém o grafo de
//! dependências com uma seta só entre cada par de camadas.

#![forbid(unsafe_code)]
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
pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod event;
pub mod queries;
pub mod services;

pub use bootstrap::app_provider::AppProvider;
pub use config::AppSecrets;
pub use event::{MetaEvent, MetaEventStackSubscriber};

// --- Reexports para a apresentação -----------------------------------------

/// Os objetos de domínio que um caso de uso devolve.
///
/// Read-only e mapeados imediatamente para o fio — é a exceção de borda que a
/// DI estática admite, e a única razão de haver `Box<dyn>` no sistema.
pub mod domain {
    pub use portmaster_domain::domain::Container;
    pub use portmaster_domain::domain::Product;
    pub use portmaster_domain::domain::Role;
    pub use portmaster_domain::domain::User;
    pub use portmaster_domain::domain::{ManifestCargo, ManifestChange};
    pub use portmaster_domain::enums::{ContainerStatus, RiskClass, TelemetryEvent};
    pub use portmaster_domain::error::FieldError;
}

/// Os read models do lado de leitura.
pub use portmaster_infra::query::views;

/// O log estruturado.
///
/// O [`SystemLogger`] é o global, para os pontos sem construtor onde injetar.
pub use portmaster_infra::logging::{Logger, LoggerFactory, SystemLogger};

/// Os geradores de id que não são identidade de entidade.
///
/// Nascem no `domain` — emitir id é contrato de negócio — e passam por aqui
/// porque quem os usa é a apresentação: o refresh token opaco e o `request_id`.
/// O gerador de identidade de entidade **não** atravessa: com ele nas mãos, o
/// `api-http` conseguiria nomear uma linha sem passar pelo `TableModule`.
pub use portmaster_domain::id::{RandomIdGenerator, SequentialIdGenerator};

/// Os segredos das camadas de baixo.
///
/// Reexportados porque quem os **preenche** é a apresentação, e ela conhece só
/// esta camada. Sem isto, o `main` teria que declarar `portmaster-infra` como
/// dependência para nomear um tipo que só repassa — e a seta a mais no grafo
/// valeria para sempre, por uma struct de configuração.
pub use portmaster_domain::DomainSecrets;
pub use portmaster_infra::config::{DatabaseSslMode, InfraSecrets};

/// O texto secreto que a URI de conexão carrega.
///
/// Vem do `secrecy`: não implementa `Debug` nem `Display` úteis, o que impede a
/// senha do banco de sair num log por acidente.
pub use portmaster_infra::config::SecretString;

#[cfg(test)]
mod tests;
