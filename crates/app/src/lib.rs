//! # portmaster-app
//!
//! Orquestração. Depende de `domain` (`TableModules`, traits de objeto) e de
//! `infra` (repositories, `UnitOfWork`, cache).
//!
//! Dono dos **Command DTOs**, dos **Query DTOs** e dos traits de `UseCase` — que
//! são exatamente os ports de que a apresentação precisa, e por isso são
//! definidos aqui: o `api-http` conhece só esta camada.
//!
//! É aqui que a autorização acontece. Cada `UseCase` protegido declara no
//! construtor a permissão que exige e a confere na primeira linha, contra o
//! `UserContext` que veio no Command — o contexto chega por argumento, nunca de
//! um estado global. É também o único lugar que abre e fecha transação.
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

pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod provider;
pub mod queries;
pub mod register;
pub mod security;
pub mod services;

pub(crate) mod cache;
pub(crate) mod interno;
pub(crate) mod transaction;

pub use config::AppSecrets;
pub use provider::AppProvider;
pub use register::register;

// --- Reexports para a apresentação -----------------------------------------

/// Os objetos de domínio que um caso de uso devolve.
///
/// Read-only e mapeados imediatamente para o fio — é a exceção de borda que a
/// DI estática admite, e a única razão de haver `Box<dyn>` no sistema.
pub mod domain {
    pub use portmaster_domain::enums::{ContainerStatus, RiskClass, TelemetryEvent};
    pub use portmaster_domain::error::FieldError;
    pub use portmaster_domain::models::Container;
    pub use portmaster_domain::models::Product;
    pub use portmaster_domain::models::Role;
    pub use portmaster_domain::models::User;
    pub use portmaster_domain::models::{ManifestCargo, ManifestChange};
}

/// Os read models do lado de leitura.
pub use portmaster_infra::query::views;

/// O log estruturado.
///
/// O [`SystemLogger`] é o global, para os pontos sem construtor onde injetar.
pub use portmaster_infra::logging::{Logger, LoggerFactory, SystemLogger};

/// A hora corrente, injetada.
pub use portmaster_infra::clock::Clock;

/// Os geradores de id que não são identidade de entidade.
///
/// Vivem na `infra` e passam por aqui: o refresh token opaco e o `request_id`
/// são da apresentação, não do domínio.
pub use portmaster_infra::id::{RandomIdGenerator, SortableIdGenerator};

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
mod tests {
    use super::*;
    use crate::commands::session::LoginCommand;
    use crate::error::AppError;
    use crate::queries::product::ListProductsQuery;
    use crate::services::{ProductUseCase, SessionUseCase};
    use std::sync::Arc;

    /// Reproduz o que o `api-http` fará com o provider.
    ///
    /// É o teste que mais importa nesta camada, e ele não roda nada: se compila,
    /// passou. A DI é 100% estática, então `Send` do futuro de um caso de uso é
    /// **inferido** do tipo concreto — e a inferência só acontece porque tudo
    /// aqui é genérico, sem `dyn`. Se um port passasse a segurar algo `!Send`
    /// através de um `.await`, todo handler do axum deixaria de compilar, e o
    /// erro apareceria lá — a três camadas de distância da causa.
    ///
    /// Ver tmp/architecture/lifetimes-auto-traits.md: o bound de `Send` mora no
    /// ponto de uso, que é exatamente isto.
    #[allow(dead_code, reason = "o teste é a compilação: nunca é chamado")]
    /// O `tokio::spawn` no corpo é o que prova o ponto: ele exige
    /// `Send + 'static`, ou seja, que o futuro do caso de uso atravesse uma
    /// fronteira de execução como atravessaria num handler.
    async fn o_provider_serve_um_handler_do_axum<P>(provider: Arc<P>) -> Result<(), AppError>
    where
        P: AppProvider + Send + Sync + 'static,
    {
        let context = context::UserContext {
            id: "1".into(),
            name: "Ana".into(),
            email: "ana@portmaster.local".into(),
            roles: Vec::new(),
        };

        let handle = tokio::spawn(async move {
            provider
                .product_use_case()
                .list(ListProductsQuery {
                    context,
                    cursor: None,
                    limit: None,
                    search: None,
                })
                .await
        });

        handle.await.expect("a tarefa não deve entrar em pânico")?;

        Ok(())
    }

    /// O mesmo, para um caso de uso de escrita — que devolve `Box<dyn …>`.
    ///
    /// A exceção de borda do `dyn` não pode custar a `Send`-ness do retorno,
    /// senão o objeto de domínio não atravessaria até a apresentação.
    #[allow(dead_code, reason = "o teste é a compilação: nunca é chamado")]
    async fn a_escrita_tambem_atravessa_uma_tarefa<P>(provider: Arc<P>) -> Result<(), AppError>
    where
        P: AppProvider + Send + Sync + 'static,
    {
        let handle = tokio::spawn(async move {
            provider
                .session_use_case()
                .login(LoginCommand {
                    email: "ana@portmaster.local".into(),
                    password: "Portmaster1".into(),
                })
                .await
        });

        let user = handle.await.expect("a tarefa não deve entrar em pânico")?;

        // O trait reexportado é o que a apresentação usa para mapear ao fio.
        let _: &str = user.id();

        Ok(())
    }
}
