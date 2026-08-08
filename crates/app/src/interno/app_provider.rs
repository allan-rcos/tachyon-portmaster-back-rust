//! A implementação do provider da aplicação.

use portmaster_domain::DomainProvider;
use portmaster_infra::id::{RandomIdGenerator, SortableIdGenerator};
use portmaster_infra::logging::LoggerFactory;
use portmaster_infra::InfraProvider;

use crate::provider::AppProvider;
use crate::services::interno::account_use_case_impl::AccountUseCaseImpl;
use crate::services::interno::container_use_case_impl::ContainerUseCaseImpl;
use crate::services::interno::manifest_use_case_impl::ManifestUseCaseImpl;
use crate::services::interno::mark_use_case_impl::MarkUseCaseImpl;
use crate::services::interno::metadata_use_case_impl::MetadataUseCaseImpl;
use crate::services::interno::metrics_use_case_impl::MetricsUseCaseImpl;
use crate::services::interno::product_use_case_impl::ProductUseCaseImpl;
use crate::services::interno::role_use_case_impl::RoleUseCaseImpl;
use crate::services::interno::session_use_case_impl::SessionUseCaseImpl;
use crate::services::interno::user_use_case_impl::UserUseCaseImpl;
use crate::services::{
    AccountUseCase, ContainerUseCase, ManifestUseCase, MarkUseCase, MetadataUseCase,
    MetricsUseCase, ProductUseCase, RoleUseCase, SessionUseCase, UserUseCase,
};

/// A implementação do provider. Privada: nenhum crate exporta impl.
pub(crate) struct AppProviderImpl<D, I> {
    /// O provider do `domain`, de onde saem os `TableModules`.
    domain: D,
    /// O provider da `infra`, de onde saem repositories, cache e leitura.
    infra: I,
}

impl<D, I> AppProviderImpl<D, I> {
    /// Guarda os subproviders das camadas de baixo.
    ///
    /// Não há recurso caro aqui: o pool e os caches nasceram no `register` da
    /// `infra` e chegam dentro do `I`.
    pub(crate) const fn new(domain: D, infra: I) -> Self {
        Self { domain, infra }
    }
}

impl<D: DomainProvider, I: InfraProvider> AppProvider for AppProviderImpl<D, I> {
    fn account_use_case(&self) -> impl AccountUseCase {
        AccountUseCaseImpl::new(
            self.infra.user_repository(),
            self.domain.user_table_module(),
            self.domain.auth_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn container_use_case(&self) -> impl ContainerUseCase {
        ContainerUseCaseImpl::new(
            self.infra.container_repository(),
            self.domain.container_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn manifest_use_case(&self) -> impl ManifestUseCase {
        ManifestUseCaseImpl::new(
            self.infra.container_repository(),
            self.infra.product_repository(),
            self.infra.manifest_repository(),
            self.domain.manifest_table_module(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn mark_use_case(&self) -> impl MarkUseCase {
        MarkUseCaseImpl::new(
            self.domain.marker_table_module(),
            self.infra.marker_repository(),
        )
    }

    fn metadata_use_case(&self) -> impl MetadataUseCase {
        MetadataUseCaseImpl::new(self.infra.permission_repository())
    }

    fn metrics_use_case(&self) -> impl MetricsUseCase {
        MetricsUseCaseImpl::new(
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn product_use_case(&self) -> impl ProductUseCase {
        ProductUseCaseImpl::new(
            self.infra.product_repository(),
            self.domain.product_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn role_use_case(&self) -> impl RoleUseCase {
        RoleUseCaseImpl::new(
            self.infra.role_repository(),
            self.domain.role_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn session_use_case(&self) -> impl SessionUseCase {
        SessionUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.infra.permission_repository(),
            self.domain.user_table_module(),
            self.domain.role_table_module(),
            self.domain.auth_table_module(),
            self.infra.unit_of_work(),
        )
    }

    fn user_use_case(&self) -> impl UserUseCase {
        UserUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.domain.user_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn logger_factory(&self) -> impl LoggerFactory {
        self.infra.logger_factory()
    }

    fn random_id_generator(&self) -> impl RandomIdGenerator {
        self.infra.random_id_generator()
    }

    fn sortable_id_generator(&self) -> impl SortableIdGenerator {
        self.infra.sortable_id_generator()
    }
}
