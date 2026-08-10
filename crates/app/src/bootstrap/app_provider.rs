//! A implementação do provider da aplicação.

use portmaster_domain::id::{RandomIdGenerator, SequentialIdGenerator};
use portmaster_domain::DomainProvider;
use portmaster_infra::logging::LoggerFactory;
use portmaster_infra::InfraProvider;

use crate::bootstrap::provider::AppProvider;
use crate::services::intern::account_use_case_impl::AccountUseCaseImpl;
use crate::services::intern::container_use_case_impl::ContainerUseCaseImpl;
use crate::services::intern::manifest_use_case_impl::ManifestUseCaseImpl;
use crate::services::intern::mark_use_case_impl::MarkUseCaseImpl;
use crate::services::intern::metadata_use_case_impl::MetadataUseCaseImpl;
use crate::services::intern::metrics_use_case_impl::MetricsUseCaseImpl;
use crate::services::intern::product_use_case_impl::ProductUseCaseImpl;
use crate::services::intern::role_use_case_impl::RoleUseCaseImpl;
use crate::services::intern::session_use_case_impl::SessionUseCaseImpl;
use crate::services::intern::user_use_case_impl::UserUseCaseImpl;
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
    fn account_use_case(&self) -> impl AccountUseCase + Clone + use<D, I> + 'static {
        AccountUseCaseImpl::new(
            self.infra.user_repository(),
            self.domain.user_table_module(),
            self.domain.auth_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn container_use_case(&self) -> impl ContainerUseCase + Clone + use<D, I> + 'static {
        ContainerUseCaseImpl::new(
            self.infra.container_repository(),
            self.domain.container_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn manifest_use_case(&self) -> impl ManifestUseCase + Clone + use<D, I> + 'static {
        ManifestUseCaseImpl::new(
            self.infra.container_repository(),
            self.infra.product_repository(),
            self.infra.manifest_repository(),
            self.domain.manifest_table_module(),
            self.infra.view_cache_repository(),
        )
    }

    fn mark_use_case(&self) -> impl MarkUseCase + Clone + use<D, I> + 'static {
        MarkUseCaseImpl::new(
            self.domain.marker_table_module(),
            self.domain.marker_group_table_module(),
            self.infra.marker_repository(),
            self.infra.marker_group_repository(),
        )
    }

    fn metadata_use_case(&self) -> impl MetadataUseCase + Clone + use<D, I> + 'static {
        MetadataUseCaseImpl::new(
            self.infra.permission_repository(),
            self.domain.permission_table_module(),
        )
    }

    fn metrics_use_case(&self) -> impl MetricsUseCase + Clone + use<D, I> + 'static {
        MetricsUseCaseImpl::new(
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn product_use_case(&self) -> impl ProductUseCase + Clone + use<D, I> + 'static {
        ProductUseCaseImpl::new(
            self.infra.product_repository(),
            self.domain.product_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn role_use_case(&self) -> impl RoleUseCase + Clone + use<D, I> + 'static {
        RoleUseCaseImpl::new(
            self.infra.role_repository(),
            self.domain.role_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn session_use_case(&self) -> impl SessionUseCase + Clone + use<D, I> + 'static {
        SessionUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.infra.permission_repository(),
            self.domain.user_table_module(),
            self.domain.role_table_module(),
            self.domain.auth_table_module(),
        )
    }

    fn user_use_case(&self) -> impl UserUseCase + Clone + use<D, I> + 'static {
        UserUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.domain.user_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn logger_factory(&self) -> impl LoggerFactory + use<D, I> + 'static {
        self.infra.logger_factory()
    }

    fn random_id_generator(&self) -> impl RandomIdGenerator + use<D, I> {
        self.domain.random_id_generator()
    }

    fn sequential_id_generator(&self) -> impl SequentialIdGenerator + use<D, I> {
        self.domain.sequential_id_generator()
    }
}
