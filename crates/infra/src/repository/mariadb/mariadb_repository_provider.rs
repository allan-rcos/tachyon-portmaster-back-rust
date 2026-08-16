//! Quem serve os repositórios sobre o `MariaDB`.

use crate::repository::mariadb::container_repository::container_repository;
use crate::repository::mariadb::manifest_repository::manifest_repository;
use crate::repository::mariadb::product_repository::product_repository;
use crate::repository::mariadb::role_repository::role_repository;
use crate::repository::mariadb::user_repository::user_repository;
use crate::repository::{
    ContainerRepository, ManifestRepository, ProductRepository, RoleRepository, UserRepository,
};
use crate::scope::ScopeProvider;

/// Os repositórios que falam SQL.
///
/// Estático e sem estado: cada factory pega o handle do banco do
/// [`ScopeProvider`] e monta um repositório novo. Nada é guardado num
/// `OnceLock` — um repositório é o handle mais nada, e o handle é que é único.
///
/// Todos devolvem `Result` porque todos dependem do pool, e o pool precisa dos
/// segredos antes da primeira criação. As chamadas acontecem no boot, dentro de
/// funções que já devolvem `anyhow::Result`; o `?` não alcança nenhum caminho
/// de requisição.
pub(crate) struct MariaDbRepositoryProvider;

impl MariaDbRepositoryProvider {
    /// A persistência de usuários, já ligada à de papéis.
    pub(crate) fn user() -> anyhow::Result<impl UserRepository + Sync + Clone + use<> + 'static> {
        Ok(user_repository(Self::role()?, ScopeProvider::database()?))
    }

    /// A persistência de papéis.
    pub(crate) fn role() -> anyhow::Result<impl RoleRepository + Sync + Clone + use<> + 'static> {
        Ok(role_repository(ScopeProvider::database()?))
    }

    /// A persistência de produtos.
    pub(crate) fn product(
    ) -> anyhow::Result<impl ProductRepository + Sync + Clone + use<> + 'static> {
        Ok(product_repository(ScopeProvider::database()?))
    }

    /// A persistência de contêineres.
    pub(crate) fn container(
    ) -> anyhow::Result<impl ContainerRepository + Sync + Clone + use<> + 'static> {
        Ok(container_repository(ScopeProvider::database()?))
    }

    /// A persistência de manifesto.
    pub(crate) fn manifest(
    ) -> anyhow::Result<impl ManifestRepository + Sync + Clone + use<> + 'static> {
        Ok(manifest_repository(ScopeProvider::database()?))
    }
}
