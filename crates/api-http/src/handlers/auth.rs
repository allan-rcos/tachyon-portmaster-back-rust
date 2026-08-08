//! `/setup` e `/auth/*` — o começo e o fim de uma sessão.
//!
//! É o único módulo de handler que conhece token e cookie. Os casos de uso
//! atrás dele não sabem o que é um JWT: o `app` confere credenciais e devolve um
//! usuário, e transformar aquilo numa sessão é trabalho desta camada.
//!
//! ## O refresh é rotativo e revogável
//!
//! O access token é auto-contido e vale até expirar — nada o cancela, e é por
//! isso que ele dura pouco. O refresh faz o contrário: é opaco, não afirma nada
//! sozinho, e sua validade é uma **marca** que pode ser apagada. Um refresh gasto
//! é invalidado no mesmo instante em que o sucessor é emitido, então apresentá-lo
//! de novo — por um cliente confuso ou por quem o roubou — falha.
//!
//! A rotação relê o usuário no banco de propósito. Copiar os papéis do token que
//! está vencendo faria uma permissão revogada sobreviver a cada renovação, e uma
//! sessão longa nunca perderia o que já lhe foi tirado.
//!
//! ## Uma recusa aqui limpa os cookies
//!
//! Sem isso o cliente reapresenta o token morto a cada tentativa e nunca volta ao
//! login. É a razão de [`ApiError`] carregar cookies.

use axum::http::{HeaderMap, StatusCode};
use portmaster_app::commands::marker::SetMarkerCommand;
use portmaster_app::commands::session::LoginCommand;
use portmaster_app::commands::session::SetupCommand;
use portmaster_app::context::UserContext;
use portmaster_app::domain::User;
use portmaster_app::queries::marker::GetMarkerQuery;
use portmaster_app::security::REFRESH_TOKEN_GROUP;
use portmaster_app::services::MarkUseCase;
use portmaster_app::services::SessionUseCase;
use portmaster_app::RandomIdGenerator;

use crate::cookie::AuthCookie;
use crate::error::api_error::ApiError;
use crate::token::refresh_token::RefreshToken;
use crate::token::token_service::TokenService;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::auth::login_request_factory::LoginRequestFactory;
use crate::wire::dto::auth::login_response_factory::LoginResponseFactory;
use crate::wire::dto::auth::setup_request_factory::SetupRequestFactory;
use crate::wire::dto::auth::user_response_factory::UserResponseFactory;
use crate::wire::no_content::NoContent;
use crate::wire::wire::Wire;

/// O que o `token_type` da resposta declara.
///
/// Não é `Bearer`: o token não viaja em `Authorization`, e sim num cookie
/// `HttpOnly` que o cliente nunca lê. Dizer `Bearer` sugeriria um uso que a API
/// não aceita.
const TOKEN_TYPE: &str = "cookie";

/// Os handlers de sessão.
pub struct AuthHandlers<S, M, R> {
    sessions: S,
    marks: M,
    random: R,
    tokens: TokenService,
    cookies: AuthCookie,
    refresh_ttl_seconds: u64,
}

impl<S, M, R> AuthHandlers<S, M, R>
where
    S: SessionUseCase,
    M: MarkUseCase,
    R: RandomIdGenerator,
{
    /// Monta os handlers.
    pub(crate) const fn new(
        sessions: S,
        marks: M,
        random: R,
        tokens: TokenService,
        cookies: AuthCookie,
        refresh_ttl_seconds: u64,
    ) -> Self {
        Self {
            sessions,
            marks,
            random,
            tokens,
            cookies,
            refresh_ttl_seconds,
        }
    }

    /// `POST /setup`
    ///
    /// Pública, e fechada para sempre depois do primeiro usuário — quem pergunta
    /// depois recebe 409. Responde `201` **já logado**: obrigar quem acabou de
    /// digitar a senha a chamar `/auth/login` em seguida seria cerimônia.
    pub(crate) async fn setup(
        &self,
        wire: Wire,
        Body(request): Body<SetupRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let user = self
            .sessions
            // `unwrap_or_default` e não um erro aqui: campo ausente vira string
            // vazia, e é o `TableModule` que a recusa — nomeando **todos** os
            // campos que faltaram, de uma vez. Levantar erro nesta camada
            // devolveria um problema por requisição e duplicaria, no `api-http`,
            // uma regra que já mora no `domain`.
            .setup(SetupCommand {
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
                password: request.password.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        self.issue_session(wire, user.as_ref(), true).await
    }

    /// `POST /auth/login`
    ///
    /// Pública. E-mail desconhecido e senha errada respondem igual — é o `app`
    /// que garante isso, e repetir a decisão aqui daria ao sistema duas opiniões
    /// sobre a mesma coisa.
    pub(crate) async fn login(
        &self,
        wire: Wire,
        Body(request): Body<LoginRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let user = self
            .sessions
            // Credencial ausente vira string vazia e falha como credencial
            // errada: o `app` responde igual para e-mail desconhecido e senha
            // errada, e distinguir "não mandou o campo" aqui entregaria ao
            // atacante metade da resposta.
            .login(LoginCommand {
                email: request.email.unwrap_or_default(),
                password: request.password.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        self.issue_session(wire, user.as_ref(), false).await
    }

    /// `POST /auth/refresh`
    ///
    /// Pública no sentido de não exigir access token: quem chega aqui está
    /// justamente com um que expirou. O que ela exige é o refresh, e ele vale
    /// uma vez só.
    pub(crate) async fn refresh(&self, headers: HeaderMap) -> Result<NoContent, ApiError> {
        let Some(presented) = self.cookies.read_refresh(&headers) else {
            // Sem cookie não há o que limpar, e limpar assim mesmo mandaria um
            // `Set-Cookie` para quem nunca teve sessão.
            return Err(refused());
        };

        let user = self.rotate(&presented).await.map_err(|error| {
            // O token apresentado está morto. Tirá-lo do navegador é o que
            // impede o cliente de reapresentá-lo para sempre.
            error
                .with_cookie(self.cookies.clear_access())
                .with_cookie(self.cookies.clear_refresh())
        })?;

        let refreshed = self.mint_refresh(user.as_ref()).await?;
        let access = self.tokens.issue(user.as_ref())?;

        Ok(NoContent::new()
            .with_cookie(self.cookies.issue_access(&access))
            .with_cookie(self.cookies.issue_refresh(&refreshed)))
    }

    /// `POST /auth/logout`
    ///
    /// Revoga o refresh e limpa os dois cookies. O access token continua
    /// tecnicamente válido até expirar — é o preço de ele ser auto-contido — mas
    /// sai do navegador aqui, e sem refresh a sessão não se renova.
    pub(crate) async fn logout(&self, headers: HeaderMap) -> Result<NoContent, ApiError> {
        if let Some(presented) = self.cookies.read_refresh(&headers) {
            // Esforço, não condição: falhar em revogar não pode impedir o
            // cliente de sair. O que fica é o registro para quem investiga.
            if let Err(error) = self.revoke(&presented).await {
                tracing::warn!(error = %error, "não foi possível revogar o refresh token no logout");
            }
        }

        Ok(NoContent::new()
            .with_cookie(self.cookies.clear_access())
            .with_cookie(self.cookies.clear_refresh()))
    }

    /// Emite access e refresh, e monta o corpo da sessão.
    async fn issue_session(
        &self,
        wire: Wire,
        user: &dyn User,
        created: bool,
    ) -> Result<ApiResponse, ApiError> {
        let refresh = self.mint_refresh(user).await?;
        let access = self.tokens.issue(user)?;

        let body =
            LoginResponseFactory::new(access.clone(), TOKEN_TYPE, UserResponseFactory::of(user));

        let response = if created {
            ApiResponse::created(wire, body)
        } else {
            ApiResponse::ok(wire, body)
        };

        Ok(response
            .with_cookie(self.cookies.issue_access(&access))
            .with_cookie(self.cookies.issue_refresh(&refresh)))
    }

    /// Gasta o refresh apresentado e devolve o usuário que ele nomeia.
    ///
    /// A ordem importa: o token só é invalidado **depois** de o usuário ser
    /// reencontrado. Invalidar antes queimaria a sessão de quem apresentou um
    /// token bom num momento em que o banco estava fora.
    async fn rotate(&self, presented: &str) -> Result<Box<dyn User>, ApiError> {
        let Some(owner) = RefreshToken::owner_of(presented) else {
            return Err(refused());
        };

        let valid = self
            .marks
            .is_valid(GetMarkerQuery {
                group: REFRESH_TOKEN_GROUP.to_owned(),
                value: presented.to_owned(),
            })
            .await
            .map_err(ApiError::of_app)?;

        if !valid {
            return Err(refused());
        }

        // Só o id importa: o que este contexto faz é dizer **quem** reler. Os
        // papéis vêm do banco logo em seguida, e é essa releitura que faz uma
        // permissão revogada não sobreviver à renovação.
        let user = self
            .sessions
            .validate(&UserContext {
                id: owner.to_owned(),
                name: String::new(),
                email: String::new(),
                roles: Vec::new(),
            })
            .await
            .map_err(|_| refused())?;

        self.revoke(presented).await.map_err(ApiError::of_app)?;

        Ok(user)
    }

    /// Emite um refresh novo para um usuário e o marca como válido.
    async fn mint_refresh(&self, user: &dyn User) -> Result<String, ApiError> {
        let token = RefreshToken::issue(user.id(), &self.random.next());

        self.marks
            .set(SetMarkerCommand {
                group: REFRESH_TOKEN_GROUP.to_owned(),
                value: token.clone(),
                flag: true,
                ttl_seconds: self.refresh_ttl_seconds,
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(token)
    }

    /// Desliga a marca de um refresh.
    async fn revoke(&self, token: &str) -> Result<(), portmaster_app::error::AppError> {
        self.marks
            .set(SetMarkerCommand {
                group: REFRESH_TOKEN_GROUP.to_owned(),
                value: token.to_owned(),
                flag: false,
                ttl_seconds: self.refresh_ttl_seconds,
            })
            .await
    }
}

/// A recusa de um refresh, seja qual for o motivo.
///
/// Cookie ausente, formato errado, marca apagada, usuário removido — todos
/// respondem igual. Distinguir diria a quem tentasse adivinhar em que ponto do
/// caminho ele chegou, e para o cliente legítimo a ação é a mesma: entrar de
/// novo.
fn refused() -> ApiError {
    ApiError::new(
        StatusCode::UNAUTHORIZED,
        "Invalid or expired refresh token.",
    )
}
