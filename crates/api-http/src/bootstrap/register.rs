//! O boot da apresentação.

use portmaster_app::AppProvider;

use crate::bootstrap::api_provider::ApiProviderImpl;
use crate::bootstrap::provider::ApiProvider;
use crate::config::api_config::ApiConfig;
use crate::config::jwt_config::JwtConfig;
use crate::cookie::intern::http_auth_cookie::HttpAuthCookie;
use crate::token::intern::jwt_token_service::JwtTokenService;

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
/// ## A configuração morre aqui
///
/// A [`ApiConfig`] e a [`JwtConfig`] entram por valor, são destrinchadas nos
/// valores que cada construtor precisa — o serviço de token pega o segredo e o
/// TTL, os cookies pegam nomes e política — e saem de escopo. Nada as guarda, e
/// nenhum objeto do sistema carrega um objeto de configuração para consultar
/// depois.
pub(crate) fn register<P: AppProvider>(
    app: P,
    config: ApiConfig,
    jwt: &JwtConfig,
) -> impl ApiProvider {
    let refresh_ttl_seconds = jwt.refresh_ttl.as_secs();
    let tokens = JwtTokenService::new(jwt);
    let cookies = HttpAuthCookie::new(jwt);

    ApiProviderImpl::new(
        app,
        tokens,
        cookies,
        config.environment,
        refresh_ttl_seconds,
        config.request_timeout,
        config.cors_origins,
    )
}
