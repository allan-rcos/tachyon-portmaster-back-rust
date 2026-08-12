//! A implementação do provider da aplicação.

use portmaster_domain::id::{RandomIdGenerator, SequentialIdGenerator};
use portmaster_domain::DomainProvider;
use portmaster_infra::logging::LoggerFactory;
use portmaster_infra::InfraProvider;

use crate::bootstrap::provider::AppProvider;
use crate::services::intern::account_service_impl::AccountServiceImpl;
use crate::services::intern::container_service_impl::ContainerServiceImpl;
use crate::services::intern::manifest_service_impl::ManifestServiceImpl;
use crate::services::intern::mark_service_impl::MarkServiceImpl;
use crate::services::intern::metadata_service_impl::MetadataServiceImpl;
use crate::services::intern::metrics_service_impl::MetricsServiceImpl;
use crate::services::intern::product_service_impl::ProductServiceImpl;
use crate::services::intern::role_service_impl::RoleServiceImpl;
use crate::services::intern::session_service_impl::SessionServiceImpl;
use crate::services::intern::user_service_impl::UserServiceImpl;
use crate::services::{
    AccountService, ContainerService, ManifestService, MarkService, MetadataService,
    MetricsService, ProductService, RoleService, SessionService, UserService,
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
    fn account_service(&self) -> impl AccountService + Clone + use<D, I> + 'static {
        AccountServiceImpl::new(
            self.infra.user_repository(),
            self.domain.user_table_module(),
            self.domain.auth_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn container_service(&self) -> impl ContainerService + Clone + use<D, I> + 'static {
        ContainerServiceImpl::new(
            self.infra.container_repository(),
            self.domain.container_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn manifest_service(&self) -> impl ManifestService + Clone + use<D, I> + 'static {
        ManifestServiceImpl::new(
            self.infra.container_repository(),
            self.infra.product_repository(),
            self.infra.manifest_repository(),
            self.domain.manifest_table_module(),
            self.infra.view_cache_repository(),
        )
    }

    fn mark_service(&self) -> impl MarkService + Clone + use<D, I> + 'static {
        MarkServiceImpl::new(
            self.domain.marker_table_module(),
            self.domain.marker_group_table_module(),
            self.infra.marker_repository(),
            self.infra.marker_group_repository(),
        )
    }

    fn metadata_service(&self) -> impl MetadataService + Clone + use<D, I> + 'static {
        MetadataServiceImpl::new(
            self.infra.permission_repository(),
            self.domain.permission_table_module(),
        )
    }

    fn metrics_service(&self) -> impl MetricsService + Clone + use<D, I> + 'static {
        MetricsServiceImpl::new(
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn product_service(&self) -> impl ProductService + Clone + use<D, I> + 'static {
        ProductServiceImpl::new(
            self.infra.product_repository(),
            self.domain.product_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn role_service(&self) -> impl RoleService + Clone + use<D, I> + 'static {
        RoleServiceImpl::new(
            self.infra.role_repository(),
            self.domain.role_table_module(),
            self.infra.query_repository(),
            self.infra.view_cache_repository(),
        )
    }

    fn session_service(&self) -> impl SessionService + Clone + use<D, I> + 'static {
        SessionServiceImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.infra.permission_repository(),
            self.domain.user_table_module(),
            self.domain.role_table_module(),
            self.domain.auth_table_module(),
        )
    }

    fn user_service(&self) -> impl UserService + Clone + use<D, I> + 'static {
        UserServiceImpl::new(
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
