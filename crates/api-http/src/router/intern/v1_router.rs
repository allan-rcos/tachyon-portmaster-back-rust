//! A primeira versão publicada do contrato REST.

use crate::bootstrap::api_provider::ApiProvider;
use crate::controllers::{
    account_routes, auth_routes, container_routes, manifest_routes, metadata_routes,
    metrics_routes, product_routes, role_routes, server_routes, user_routes,
};
use crate::router::route::Route;
use crate::router::versioned_router::VersionedRouter;

/// A v1, montada sob `/v1` e — enquanto for a mais nova — também na raiz.
///
/// **Esta tabela é congelada.** Uma mudança no que qualquer uma destas rotas
/// significa é uma `V2Router` ao lado deste arquivo, não uma edição nele — é o
/// ponto inteiro de a versão ser um tipo.
///
/// Ela não lista caminho nenhum. Cada recurso traz as próprias rotas, e o que
/// este arquivo faz é dizer **quais recursos** a v1 publica: o encanamento de
/// extractor mora ao lado do controller que o consome, e aqui não aparece nome
/// de tipo interno nenhum.
pub(crate) struct V1Router;

impl VersionedRouter for V1Router {
    const VERSION: u16 = 1;

    fn routes() -> anyhow::Result<Vec<Route>> {
        Ok([
            server_routes::routes(ApiProvider::server_controller()),
            auth_routes::routes(ApiProvider::auth_controller()?),
            account_routes::routes(ApiProvider::account_controller()?),
            product_routes::routes(ApiProvider::product_controller()?),
            role_routes::routes(ApiProvider::role_controller()?),
            container_routes::routes(ApiProvider::container_controller()?),
            manifest_routes::routes(ApiProvider::manifest_controller()?),
            user_routes::routes(ApiProvider::user_controller()?),
            metadata_routes::routes(ApiProvider::metadata_controller()),
            metrics_routes::routes(ApiProvider::metrics_controller()?),
        ]
        .into_iter()
        .flatten()
        .collect())
    }
}
