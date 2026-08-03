//! # portmaster-domain
//!
//! O núcleo. Só regras de negócio: não depende de nenhum outro crate e não toca
//! banco, rede ou I/O.
//!
//! Exporta **apenas traits** — os traits de objeto de domínio (read-only), os
//! TableModules, a trait [`DomainProvider`] e [`register`]. Models,
//! implementações de TableModule e helpers internos (gerador de id, hashers) são
//! privados ao crate e servidos pelos factories do provider.
//!
//! Essa reserva não é cerimônia. Se o `app` alcançasse o hasher de senha, ele
//! conseguiria gravar um usuário direto na `infra`, pulando a validação do
//! TableModule — e no dia em que a regra de senha mudasse, metade do sistema
//! estaria validando pela regra antiga.
//!
//! ## Ids
//!
//! Saem daqui como `String` base62. O TableModule gera o Snowflake `i64` e o
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

pub mod auth;
pub mod container;
pub mod enums;
pub mod error;
pub mod id;
pub mod manifest;
pub mod marker;
pub mod metadata;
pub mod product;
pub mod role;
pub mod user;

pub(crate) mod security;

use auth::tm::AuthTMImpl;
use container::tm::ContainerTMImpl;
use manifest::tm::ManifestTMImpl;
use marker::MarkerTMImpl;
use metadata::marker_group::MarkerGroupTMImpl;
use metadata::permission::PermissionTMImpl;
use product::tm::ProductTMImpl;
use role::tm::RoleTMImpl;
use security::argon2::Argon2Hasher;
use security::xxhash::XxIndexHasher;
use user::tm::UserTMImpl;

use id::IntIdGenerator;

#[cfg(feature = "id-snowflake")]
use id::snowflake::SnowflakeIdGenerator;

/// Segredos de runtime do `domain`.
///
/// Só identidade de deploy. A **estratégia** de geração de id é feature de
/// build, e a era do gerador é constante — dois deploys com epochs diferentes
/// emitiriam ids que se sobrepõem no tempo. O que varia entre instâncias é
/// apenas quem elas são.
#[derive(Debug, Clone, Copy)]
pub struct DomainSecrets {
    /// Identifica o cluster na composição do Snowflake. Faixa: 0–31.
    pub cluster_id: i32,

    /// Identifica o servidor dentro do cluster. Faixa: 0–31.
    pub server_id: i32,
}

/// Os factories do `domain`.
///
/// Cada método devolve `impl Trait`: o consumidor recebe o **contrato**, nunca o
/// tipo concreto. O despacho é estático — o compilador monomorfiza o grafo
/// inteiro, então um serviço que não pode ser construído é erro de compilação,
/// não surpresa em produção.
pub trait DomainProvider {
    /// TableModule de usuário.
    fn user_table_module(&self) -> impl user::UserTM + Send + Sync;

    /// TableModule de papel.
    fn role_table_module(&self) -> impl role::RoleTM + Send + Sync;

    /// TableModule de produto.
    fn product_table_module(&self) -> impl product::ProductTM + Send + Sync;

    /// TableModule de contêiner.
    fn container_table_module(&self) -> impl container::ContainerTM + Send + Sync;

    /// TableModule de manifesto.
    fn manifest_table_module(&self) -> impl manifest::ManifestTM + Send + Sync;

    /// TableModule de autenticação.
    fn auth_table_module(&self) -> impl auth::AuthTM + Send + Sync;

    /// TableModule de permissão.
    fn permission_table_module(&self) -> impl metadata::permission::PermissionTM + Send + Sync;

    /// TableModule de grupo de marcador.
    fn marker_group_table_module(&self)
        -> impl metadata::marker_group::MarkerGroupTM + Send + Sync;

    /// TableModule de marcador.
    fn marker_table_module(&self) -> impl marker::MarkerTM + Send + Sync;
}

/// Inicializa o `domain` e devolve o seu provider.
///
/// Não depende de camada nenhuma, então é o fim da cadeia de `register`: quem o
/// chama é o `app`, antes de montar os seus casos de uso.
pub fn register(secrets: DomainSecrets) -> impl DomainProvider {
    DomainProviderImpl { secrets }
}

/// A implementação do provider. Privada: nenhum crate exporta impl.
struct DomainProviderImpl {
    secrets: DomainSecrets,
}

impl DomainProviderImpl {
    /// Serve o gerador de id de entidade.
    ///
    /// A impl é escolhida por **feature de compilação** — decisão de
    /// arquitetura, resolvida no build, sem ramo em runtime. Os parâmetros de
    /// identidade vêm dos segredos.
    fn int_id_generator(&self) -> impl IntIdGenerator {
        #[cfg(feature = "id-snowflake")]
        SnowflakeIdGenerator::new(self.secrets.cluster_id, self.secrets.server_id)
    }
}

impl DomainProvider for DomainProviderImpl {
    fn user_table_module(&self) -> impl user::UserTM {
        UserTMImpl::new(self.int_id_generator(), Argon2Hasher::new())
    }

    fn role_table_module(&self) -> impl role::RoleTM {
        RoleTMImpl::new(self.int_id_generator())
    }

    fn product_table_module(&self) -> impl product::ProductTM {
        ProductTMImpl::new(self.int_id_generator())
    }

    fn container_table_module(&self) -> impl container::ContainerTM {
        ContainerTMImpl::new(self.int_id_generator())
    }

    fn manifest_table_module(&self) -> impl manifest::ManifestTM {
        ManifestTMImpl::new()
    }

    fn auth_table_module(&self) -> impl auth::AuthTM {
        AuthTMImpl::new(Argon2Hasher::new())
    }

    fn permission_table_module(&self) -> impl metadata::permission::PermissionTM {
        PermissionTMImpl::new()
    }

    fn marker_group_table_module(&self) -> impl metadata::marker_group::MarkerGroupTM {
        MarkerGroupTMImpl::new()
    }

    fn marker_table_module(&self) -> impl marker::MarkerTM {
        MarkerTMImpl::new(XxIndexHasher::new())
    }
}
