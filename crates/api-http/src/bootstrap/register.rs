//! O boot da apresentação.

use portmaster_app::AppProvider;

use crate::bootstrap::api_provider::ApiProviderImpl;
use crate::bootstrap::provider::ApiProvider;
use crate::config::api_config::ApiConfig;
use crate::config::jwt_config::JwtConfig;

/// Monta o provider da apresentação.
///
/// Consome o provider da camada de baixo e a configuração, e devolve algo
/// pronto. É o último elo da cadeia de `register` que começa no `domain`:
/// ninguém acima disto conhece uma impl.
///
/// `pub(crate)` e não `pub`, ao contrário dos `register` das outras camadas.
/// Elas precisam ser públicas porque a camada de cima as chama; esta é a de
/// cima. Publicá-la obrigaria a publicar o [`ApiProvider`], os dez traits de
/// controller e — por tabela — todos os VOs que aparecem nas assinaturas deles,
/// espalhando pela API do crate um grafo que só o `main` ao lado consome. O que
/// sai daqui é o [`router`](crate::router()), e ele basta.
///
/// ## Uma linha, e é o ponto
///
/// Ela destrinchava a configuração em valores soltos — o segredo do token, os
/// nomes de cookie, o ambiente, o teto de tempo, as origens de CORS — e
/// construía o serviço de token e os cookies para entregá-los prontos ao
/// provider. Era o provider recebendo classe em vez de montá-la, e uma
/// configuração nova significava um argumento novo em duas assinaturas.
///
/// Agora o provider recebe **só** o provider de baixo e a configuração, e monta
/// o que precisar de onde a informação está. Esta função existe para não expor
/// o tipo concreto do provider, e mais nada.
pub(crate) fn register<P: AppProvider>(
    app: P,
    config: ApiConfig,
    jwt: JwtConfig,
) -> impl ApiProvider {
    ApiProviderImpl::new(app, config, jwt)
}
