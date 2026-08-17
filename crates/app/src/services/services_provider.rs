//! Quem serve os casos de uso.

use portmaster_domain::DomainProvider;
use portmaster_infra::InfraProvider;

use crate::event::EventProvider;
use crate::services::intern::account_service_impl::account_service;
use crate::services::intern::container_service_impl::container_service;
use crate::services::intern::manifest_service_impl::manifest_service;
use crate::services::intern::mark_service_impl::mark_service;
use crate::services::intern::metadata_service_impl::metadata_service;
use crate::services::intern::metrics_service_impl::metrics_service;
use crate::services::intern::product_service_impl::product_service;
use crate::services::intern::role_service_impl::role_service;
use crate::services::intern::session_service_impl::session_service;
use crate::services::intern::user_service_impl::user_service;
use crate::services::{
    AccountService, ContainerService, ManifestService, MarkService, MetadataService,
    MetricsService, ProductService, RoleService, SessionService, UserService,
};

/// Os services, já costurados com os ports que consomem.
///
/// É aqui que a costura mora: quais repositórios, quais `TableModules` e qual
/// cache cada caso de uso recebe.
///
/// Nada é guardado. Um service é um punhado de handles clonados, e o que precisa
/// ser único — pool, caches, gerador de id — já é único dentro dele.
pub(crate) struct ServicesProvider;

impl ServicesProvider {
    /// A conta do próprio usuário.
    pub(crate) fn account() -> anyhow::Result<impl AccountService + Sync + Clone + use<> + 'static>
    {
        Ok(account_service(
            InfraProvider::user_repository()?,
            DomainProvider::user_table_module(),
            DomainProvider::auth_table_module(),
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }

    /// Contêineres.
    pub(crate) fn container(
    ) -> anyhow::Result<impl ContainerService + Sync + Clone + use<> + 'static> {
        Ok(container_service(
            InfraProvider::container_repository()?,
            DomainProvider::container_table_module(),
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }

    /// Carga e telemetria.
    pub(crate) fn manifest() -> anyhow::Result<impl ManifestService + Sync + Clone + use<> + 'static>
    {
        Ok(manifest_service(
            InfraProvider::container_repository()?,
            InfraProvider::product_repository()?,
            InfraProvider::manifest_repository()?,
            DomainProvider::manifest_table_module(),
            InfraProvider::view_cache_repository(),
        ))
    }

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    ///
    /// Infalível: os dois repositórios que ela consome vivem em memória.
    pub(crate) fn mark() -> impl MarkService + Sync + Clone + use<> + 'static {
        mark_service(
            DomainProvider::marker_table_module(),
            DomainProvider::marker_group_table_module(),
            InfraProvider::marker_repository(),
            InfraProvider::marker_group_repository(),
        )
    }

    /// Metadados de sistema.
    ///
    /// Infalível: o catálogo de permissões vive em memória.
    pub(crate) fn metadata() -> impl MetadataService + Sync + Clone + use<> + 'static {
        metadata_service(
            InfraProvider::permission_repository(),
            DomainProvider::permission_table_module(),
        )
    }

    /// O painel do pátio.
    pub(crate) fn metrics() -> anyhow::Result<impl MetricsService + Sync + Clone + use<> + 'static>
    {
        Ok(metrics_service(
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }

    /// Produtos.
    pub(crate) fn product() -> anyhow::Result<impl ProductService + Sync + Clone + use<> + 'static>
    {
        Ok(product_service(
            InfraProvider::product_repository()?,
            DomainProvider::product_table_module(),
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }

    /// Papéis.
    pub(crate) fn role() -> anyhow::Result<impl RoleService + Sync + Clone + use<> + 'static> {
        Ok(role_service(
            InfraProvider::role_repository()?,
            DomainProvider::role_table_module(),
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }

    /// Login, validação de sessão e o setup inicial.
    pub(crate) fn session() -> anyhow::Result<impl SessionService + Sync + Clone + use<> + 'static>
    {
        Ok(session_service(
            InfraProvider::user_repository()?,
            InfraProvider::role_repository()?,
            InfraProvider::permission_repository(),
            DomainProvider::user_table_module(),
            DomainProvider::role_table_module(),
            DomainProvider::auth_table_module(),
        ))
    }

    /// Usuários.
    pub(crate) fn user() -> anyhow::Result<impl UserService + Sync + Clone + use<> + 'static> {
        Ok(user_service(
            InfraProvider::user_repository()?,
            InfraProvider::role_repository()?,
            DomainProvider::user_table_module(),
            InfraProvider::query_repository()?,
            InfraProvider::view_cache_repository(),
            EventProvider::meta_event_stack(),
        ))
    }
}
