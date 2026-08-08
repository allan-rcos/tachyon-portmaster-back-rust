//! As mensagens de `/account`.

pub(crate) mod account_profile_response_factory;
pub(crate) mod account_update_request;
pub(crate) mod account_update_request_factory;
pub(crate) mod password_change_request;
pub(crate) mod password_change_request_factory;
pub(crate) mod role_response_factory;

use crate::error::api_error::ApiError;
use crate::wire::factory::response_factory::ResponseFactory as _;
use crate::wire::tables as fbs;
use portmaster_app::views::RoleViewItem;

use role_response_factory::RoleResponseFactory;

/// Converte a lista de papéis que perfil e usuário-admin publicam igual.
///
/// As duas mensagens carregam o mesmo recorte de papel, e a conversão é a mesma
/// — duplicá-la nos dois arquivos criaria a chance de uma divergir da outra.
pub(crate) fn roles_of(
    roles: &[RoleViewItem],
) -> Result<Vec<fbs::account::RoleResponse>, ApiError> {
    roles
        .iter()
        .map(|role| RoleResponseFactory::of(role.clone()).table())
        .collect()
}
