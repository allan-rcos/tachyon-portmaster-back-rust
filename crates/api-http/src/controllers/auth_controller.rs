//! O contrato do controller de sessão.

use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::auth::login_x_request::LoginXRequest;
use crate::wire::vo::auth::login_x_response::LoginXResponse;
use crate::wire::vo::auth::setup_x_request::SetupXRequest;

/// Os handlers de sessão.
///
/// A sessão **é** um par de cookies, e quem decide o que entra neles é quem
/// emite o token — este controller. Isso não aparece na assinatura: devolver
/// `Vec<Cookie<'static>>` poria o tipo interno do crate `cookie` num contrato e
/// obrigaria cada rota a dobrar aquele vetor sobre a resposta. Ele escreve pela
/// [`CookiePort`](crate::middleware::cookie_port::CookiePort), e o middleware
/// carimba o que foi escrito.
///
/// Pela mesma razão, `refresh` e `logout` não recebem mais os cabeçalhos: eles
/// pediam a `HeaderMap` inteira porque só a impl de cookie sabia sob que nome
/// procurar. A porta sabe, então o argumento não precisa existir.
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
    async fn setup(self, request: Body<SetupXRequest>) -> ApiResponse<LoginXResponse>;

    /// `POST /auth/login`
    async fn login(self, request: Body<LoginXRequest>) -> ApiResponse<LoginXResponse>;

    /// `POST /auth/refresh`
    ///
    /// Não devolve corpo: o par novo viaja nos cookies, e a resposta é um `204`.
    async fn refresh(self) -> ApiResponse;

    /// `POST /auth/logout`
    ///
    /// Sair é sempre possível — a revogação do refresh é esforço, não condição,
    /// e o que pode falhar aqui é só escrever os cookies que apagam a sessão.
    async fn logout(self) -> ApiResponse;
}
