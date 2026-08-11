//! As rotas de metadado de sistema.

use crate::controllers::metadata_controller::MetadataController;
use crate::router::route::Route;

/// A tabela de metadado.
pub(crate) fn routes<C: MetadataController>(controller: C) -> Vec<Route> {
    vec![Route::get("/metadata/permissions", move |params| {
        controller.clone().list_permissions(params)
    })]
}
