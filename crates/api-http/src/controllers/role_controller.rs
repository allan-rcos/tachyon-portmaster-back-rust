//! O contrato do controller de papéis.

use crate::controllers::params::page_params::PageParams;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::account::role_x_response::RoleXResponse;
use crate::wire::vo::admin::role_create_x_request::RoleCreateXRequest;
use crate::wire::vo::admin::role_list_x_response::RoleListXResponse;
use crate::wire::vo::admin::role_permissions_update_x_request::RolePermissionsUpdateXRequest;
use axum::extract::{Path, Query};

/// Os handlers de papel.
///
/// Um método por rota. Cada um recebe VOs e devolve VOs — nada de axum, nada de
/// negociação de conteúdo, nada de status. Quem traduz HTTP nisto e de volta é o
/// módulo de rotas ao lado; o que está aqui dá para chamar de um teste sem subir
/// servidor nenhum, e é essa a diferença que a trait compra.
#[trait_variant::make(Send)]
pub(crate) trait RoleController: Clone + Sync + 'static {
    /// `GET /roles`
    async fn list(self, params: Query<PageParams>) -> ApiResponse<RoleListXResponse>;

    /// `POST /roles`
    async fn create(self, request: Body<RoleCreateXRequest>) -> ApiResponse<RoleXResponse>;

    /// `PUT /roles/{id}/permissions`
    async fn update_permissions(
        self,
        id: Path<String>,
        request: Body<RolePermissionsUpdateXRequest>,
    ) -> ApiResponse<RoleXResponse>;
}
