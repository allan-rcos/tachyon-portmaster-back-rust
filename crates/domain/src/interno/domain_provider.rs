//! A implementação do provider da camada.

use crate::config::DomainSecrets;
use crate::id::IntIdGenerator;
use crate::provider::DomainProvider;
use crate::security::interno::argon2_hasher::Argon2Hasher;
use crate::security::interno::xx_index_hasher::XxIndexHasher;
use crate::table_modules::interno::auth_tm_impl::AuthTMImpl;
use crate::table_modules::interno::container_tm_impl::ContainerTMImpl;
use crate::table_modules::interno::manifest_tm_impl::ManifestTMImpl;
use crate::table_modules::interno::marker_group_tm_impl::MarkerGroupTMImpl;
use crate::table_modules::interno::marker_tm_impl::MarkerTMImpl;
use crate::table_modules::interno::permission_tm_impl::PermissionTMImpl;
use crate::table_modules::interno::product_tm_impl::ProductTMImpl;
use crate::table_modules::interno::role_tm_impl::RoleTMImpl;
use crate::table_modules::interno::user_tm_impl::UserTMImpl;
use crate::table_modules::{
    AuthTM, ContainerTM, ManifestTM, MarkerGroupTM, MarkerTM, PermissionTM, ProductTM, RoleTM,
    UserTM,
};

#[cfg(feature = "id-snowflake")]
use crate::id::interno::snowflake_id_generator::SnowflakeIdGenerator;

/// A implementação do provider. Privada: nenhum crate exporta impl.
pub(crate) struct DomainProviderImpl {
    /// Quem é este servidor na composição do Snowflake.
    secrets: DomainSecrets,
}

impl DomainProviderImpl {
    /// Guarda a identidade de deploy e nada mais — não há recurso caro aqui.
    pub(crate) const fn new(secrets: DomainSecrets) -> Self {
        Self { secrets }
    }

    /// Serve o gerador de id de entidade.
    ///
    /// A impl é escolhida por **feature de compilação** — decisão de
    /// arquitetura, resolvida no build, sem ramo em runtime. Os parâmetros de
    /// identidade vêm dos segredos.
    fn int_id_generator(&self) -> impl IntIdGenerator + Clone + use<> + 'static {
        #[cfg(feature = "id-snowflake")]
        SnowflakeIdGenerator::new(self.secrets.cluster_id, self.secrets.server_id)
    }
}

impl DomainProvider for DomainProviderImpl {
    fn user_table_module(&self) -> impl UserTM + Clone + use<> + 'static {
        UserTMImpl::new(self.int_id_generator(), Argon2Hasher::new())
    }

    fn role_table_module(&self) -> impl RoleTM + Clone + use<> + 'static {
        RoleTMImpl::new(self.int_id_generator())
    }

    fn product_table_module(&self) -> impl ProductTM + Clone + use<> + 'static {
        ProductTMImpl::new(self.int_id_generator())
    }

    fn container_table_module(&self) -> impl ContainerTM + Clone + use<> + 'static {
        ContainerTMImpl::new(self.int_id_generator())
    }

    fn manifest_table_module(&self) -> impl ManifestTM + Clone + use<> + 'static {
        ManifestTMImpl::new()
    }

    fn auth_table_module(&self) -> impl AuthTM + Clone + use<> + 'static {
        AuthTMImpl::new(Argon2Hasher::new())
    }

    fn permission_table_module(&self) -> impl PermissionTM + Clone + use<> + 'static {
        PermissionTMImpl::new()
    }

    fn marker_group_table_module(&self) -> impl MarkerGroupTM + Clone + use<> + 'static {
        MarkerGroupTMImpl::new()
    }

    fn marker_table_module(&self) -> impl MarkerTM + Clone + use<> + 'static {
        MarkerTMImpl::new(XxIndexHasher::new())
    }
}
