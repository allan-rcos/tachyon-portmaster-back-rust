//! O contrato do controller de sessão.

use axum::http::HeaderMap;
use cookie::Cookie;

use crate::ports::error::api_error::ApiError;
use crate::wire::vo::auth::login_x_request::LoginXRequest;
use crate::wire::vo::auth::login_x_response::LoginXResponse;
use crate::wire::vo::auth::setup_x_request::SetupXRequest;

/// Os handlers de sessão.
///
/// É o único controller cujos métodos devolvem cookies junto com o VO, e não há
/// como ser diferente: a sessão **é** um par de cookies, e quem decide o que
/// entra neles é quem emite o token. O que a trait garante é que ele o faça sem
/// conhecer nenhuma implementação — nem a do token, nem a do cookie.
///
/// Os métodos de refresh e logout recebem os cabeçalhos em vez de um token já
/// extraído porque o nome do cookie é assunto da impl de
/// [`AuthCookie`](crate::ports::cookie::auth_cookie::AuthCookie), e as rotas não o
/// conhecem.
#[trait_variant::make(Send)]
pub(crate) trait AuthController: Clone + Sync + 'static {
    /// Declara, no boot, o grupo de marcador em que as sessões de refresh vivem.
    ///
    /// É aqui e não no `app` porque este controller é o único que marca naquele
    /// grupo, e é ele quem sabe o nome dele — a camada de aplicação é agnóstica
    /// de sessão. Precisa rodar antes da primeira requisição: o repositório de
    /// marcadores recusa marcar num grupo que não conhece.
    async fn register_marker_group(&self) -> Result<(), ApiError>;

    /// `POST /setup`
    ///
    /// Abre uma vez na vida de um deploy: cria o primeiro usuário e já o loga.
    async fn setup(
        &self,
        request: SetupXRequest,
    ) -> Result<(LoginXResponse, Vec<Cookie<'static>>), ApiError>;

    /// `POST /auth/login`
    async fn login(
        &self,
        request: LoginXRequest,
    ) -> Result<(LoginXResponse, Vec<Cookie<'static>>), ApiError>;

    /// `POST /auth/refresh`
    ///
    /// Devolve só os cookies: o access token novo viaja neles, e a resposta é um
    /// `204`.
    async fn refresh(&self, headers: HeaderMap) -> Result<Vec<Cookie<'static>>, ApiError>;

    /// `POST /auth/logout`
    ///
    /// Nunca falha: sair é sempre possível. O que ela devolve são os cookies que
    /// apagam a sessão do navegador.
    async fn logout(&self, headers: HeaderMap) -> Vec<Cookie<'static>>;
}
