//! Quem serve os `TableModules`.
//!
//! Aqui só se compõe: cada regra recebe os helpers de que precisa, e quais são
//! esses helpers é a única decisão deste arquivo.

use crate::id::IdProvider;
use crate::security::SecurityProvider;
use crate::table_modules::intern::auth_tm_impl::auth_tm;
use crate::table_modules::intern::container_tm_impl::container_tm;
use crate::table_modules::intern::manifest_tm_impl::manifest_tm;
use crate::table_modules::intern::marker_group_tm_impl::marker_group_tm;
use crate::table_modules::intern::marker_tm_impl::marker_tm;
use crate::table_modules::intern::permission_tm_impl::permission_tm;
use crate::table_modules::intern::product_tm_impl::product_tm;
use crate::table_modules::intern::role_tm_impl::role_tm;
use crate::table_modules::intern::user_tm_impl::user_tm;
use crate::table_modules::{
    AuthTM, ContainerTM, ManifestTM, MarkerGroupTM, MarkerTM, PermissionTM, ProductTM, RoleTM,
    UserTM,
};

/// Os `TableModules`, já costurados com os helpers do domínio.
///
/// Sem config: o único parâmetro de deploy do domínio é a identidade do
/// gerador, e ela é assunto do [`IdProvider`].
///
/// Nada é guardado. O que precisa ser único já é único um andar abaixo, e o que
/// sai daqui é a costura em volta dele.
pub(crate) struct TableModulesProvider;

impl TableModulesProvider {
    /// As regras de usuário.
    pub(crate) fn user() -> impl UserTM + Send + Sync + Clone + use<> + 'static {
        user_tm(IdProvider::database(), SecurityProvider::password())
    }

    /// As regras de papel.
    pub(crate) fn role() -> impl RoleTM + Send + Sync + Clone + use<> + 'static {
        role_tm(IdProvider::database())
    }

    /// As regras de produto.
    pub(crate) fn product() -> impl ProductTM + Send + Sync + Clone + use<> + 'static {
        product_tm(IdProvider::database())
    }

    /// As regras de contêiner.
    pub(crate) fn container() -> impl ContainerTM + Send + Sync + Clone + use<> + 'static {
        container_tm(IdProvider::database())
    }

    /// As regras de manifesto.
    pub(crate) fn manifest() -> impl ManifestTM + Send + Sync + Clone + use<> + 'static {
        manifest_tm()
    }

    /// As regras de autenticação.
    pub(crate) fn auth() -> impl AuthTM + Send + Sync + Clone + use<> + 'static {
        auth_tm(SecurityProvider::password())
    }

    /// As regras de permissão.
    pub(crate) fn permission() -> impl PermissionTM + Send + Sync + Clone + use<> + 'static {
        permission_tm()
    }

    /// As regras de grupo de marcador.
    pub(crate) fn marker_group() -> impl MarkerGroupTM + Send + Sync + Clone + use<> + 'static {
        marker_group_tm()
    }

    /// As regras de marcador.
    pub(crate) fn marker() -> impl MarkerTM + Send + Sync + Clone + use<> + 'static {
        marker_tm(SecurityProvider::index())
    }
}
