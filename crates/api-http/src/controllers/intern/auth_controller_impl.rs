//! O controller de sessão. Não sai do módulo.

use axum::http::{HeaderMap, StatusCode};
use cookie::Cookie;
use portmaster_app::commands::marker::{RegisterMarkerGroupCommand, SetMarkerCommand};
use portmaster_app::commands::session::{LoginCommand, SetupCommand};
use portmaster_app::context::UserContext;
use portmaster_app::domain::User;
use portmaster_app::error::{MarkerError, SessionError};
use portmaster_app::queries::marker::GetMarkerQuery;
use portmaster_app::services::{MarkUseCase, SessionUseCase};
use portmaster_app::{Logger, RandomIdGenerator};

use crate::controllers::auth_controller::AuthController;
use crate::ports::cookie::auth_cookie::AuthCookie;
use crate::ports::error::api_error::ApiError;
use crate::ports::token::refresh_token::RefreshToken;
use crate::ports::token::token_service::TokenService;
use crate::wire::vo::auth::login_x_request::LoginXRequest;
use crate::wire::vo::auth::login_x_response::LoginXResponse;
use crate::wire::vo::auth::setup_x_request::SetupXRequest;
use crate::wire::vo::auth::user_x::UserX;

/// O que o `token_type` da resposta declara.
///
/// Não é `Bearer`: o token não viaja em `Authorization`, e sim num cookie
/// `HttpOnly` que o cliente nunca lê. Dizer `Bearer` sugeriria um uso que a API
/// não aceita.
const TOKEN_TYPE: &str = "cookie";

/// O grupo de marcador em que as sessões de refresh vivem.
///
/// Mora aqui, e não no `app`: este é o único ponto do sistema que marca ou
/// consulta nesse grupo, e a camada de aplicação é agnóstica de sessão — para
/// ela um marcador é um booleano com prazo, e quem lhe dá nome é quem o usa.
const REFRESH_TOKEN_GROUP: &str = "refresh-token";

/// Os handlers de sessão, genéricos sobre tudo que consomem.
///
/// Repare que `T`, `A` e `L` são parâmetros de tipo e não structs concretas.
/// Antes o controller declarava `tokens: TokenService` e `cookies: AuthCookie`
/// com as impls, e a hierarquia que as traits desenham valia para fora do crate
/// mas não para dentro dele.
#[derive(Clone)]
pub(crate) struct AuthControllerImpl<S, M, R, T, A, L> {
    /// O caso de uso de sessão.
    sessions: S,
    /// O caso de uso de marcador, que guarda o refresh.
    marks: M,
    /// De onde sai a metade aleatória do refresh token.
    random: R,
    /// Quem emite e confere o access token.
    tokens: T,
    /// Como os cookies de sessão são escritos e lidos.
    cookies: A,
    /// Para onde vai o que não impediu a operação.
    logger: L,
    /// Por quanto tempo o refresh vale, em segundos.
    refresh_ttl_seconds: u64,
}

impl<S, M, R, T, A, L> AuthControllerImpl<S, M, R, T, A, L>
where
    S: SessionUseCase,
    M: MarkUseCase,
    R: RandomIdGenerator,
    T: TokenService,
    A: AuthCookie,
    L: Logger,
{
    /// Monta o controller.
    pub(crate) const fn new(
        sessions: S,
        marks: M,
        random: R,
        tokens: T,
        cookies: A,
        logger: L,
        refresh_ttl_seconds: u64,
    ) -> Self {
        Self {
            sessions,
            marks,
            random,
            tokens,
            cookies,
            logger,
            refresh_ttl_seconds,
        }
    }

    /// Emite access e refresh, e monta o corpo da sessão.
    async fn issue_session(
        &self,
        user: &dyn User,
    ) -> Result<(LoginXResponse, Vec<Cookie<'static>>), ApiError> {
        let refresh = self.mint_refresh(user).await?;
        let access = self.tokens.issue(user)?;

        let body = LoginXResponse {
            token: access.clone(),
            token_type: TOKEN_TYPE.to_owned(),
            user: UserX::of(user),
        };

        let cookies = vec![
            self.cookies.issue_access(&access),
            self.cookies.issue_refresh(&refresh),
        ];

        Ok((body, cookies))
    }

    /// Gasta o refresh apresentado e devolve o usuário que ele nomeia.
    ///
    /// A ordem importa: o token só é invalidado **depois** de o usuário ser
    /// reencontrado. Invalidar antes queimaria a sessão de quem apresentou um
    /// token bom num momento em que o banco estava fora.
    ///
    /// O contexto montado aqui carrega **só o id**: o que ele faz é dizer quem
    /// reler. Os papéis vêm do banco logo em seguida, e é essa releitura que faz
    /// uma permissão revogada não sobreviver à renovação.
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
            .map_err(mark_refused)?;

        if !valid {
            return Err(refused());
        }

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

        self.revoke(presented).await.map_err(mark_refused)?;

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
            .map_err(mark_refused)?;

        Ok(token)
    }

    /// Desliga a marca de um refresh.
    async fn revoke(&self, token: &str) -> Result<(), MarkerError> {
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

impl<S, M, R, T, A, L> AuthController for AuthControllerImpl<S, M, R, T, A, L>
where
    S: SessionUseCase + Clone + Send + Sync + 'static,
    M: MarkUseCase + Clone + Send + Sync + 'static,
    R: RandomIdGenerator + Clone + Send + Sync + 'static,
    T: TokenService,
    A: AuthCookie,
    L: Logger,
{
    async fn register_marker_group(&self) -> Result<(), ApiError> {
        self.marks
            .register_group(RegisterMarkerGroupCommand {
                slug: REFRESH_TOKEN_GROUP.to_owned(),
            })
            .await
            .map_err(mark_refused)
    }

    /// Cria o primeiro usuário e já o loga.
    ///
    /// Os campos entram com `unwrap_or_default` e não com erro: campo ausente
    /// vira string vazia, e é o `TableModule` que a recusa — nomeando **todos**
    /// os campos que faltaram, de uma vez. Levantar erro nesta camada devolveria
    /// um problema por requisição e duplicaria, no `api-http`, uma regra que já
    /// mora no `domain`.
    async fn setup(
        &self,
        request: SetupXRequest,
    ) -> Result<(LoginXResponse, Vec<Cookie<'static>>), ApiError> {
        let user = self
            .sessions
            .setup(SetupCommand {
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
                password: request.password.unwrap_or_default(),
            })
            .await
            .map_err(session_refused)?;

        self.issue_session(user.as_ref()).await
    }

    /// Abre a sessão.
    ///
    /// Credencial ausente vira string vazia e falha como credencial **errada**:
    /// o `app` responde igual para e-mail desconhecido e senha errada, e
    /// distinguir "não mandou o campo" aqui entregaria ao atacante metade da
    /// resposta.
    async fn login(
        &self,
        request: LoginXRequest,
    ) -> Result<(LoginXResponse, Vec<Cookie<'static>>), ApiError> {
        let user = self
            .sessions
            .login(LoginCommand {
                email: request.email.unwrap_or_default(),
                password: request.password.unwrap_or_default(),
            })
            .await
            .map_err(session_refused)?;

        self.issue_session(user.as_ref()).await
    }

    /// Troca um refresh válido por um par novo.
    ///
    /// Sem cookie não há o que limpar, e limpar assim mesmo mandaria um
    /// `Set-Cookie` para quem nunca teve sessão. Já um token apresentado e morto
    /// **é** tirado do navegador — é o que impede o cliente de reapresentá-lo
    /// para sempre.
    async fn refresh(&self, headers: HeaderMap) -> Result<Vec<Cookie<'static>>, ApiError> {
        let Some(presented) = self.cookies.read_refresh(&headers) else {
            return Err(refused());
        };

        let user = self.rotate(&presented).await.map_err(|error| {
            error
                .with_cookie(self.cookies.clear_access())
                .with_cookie(self.cookies.clear_refresh())
        })?;

        let refreshed = self.mint_refresh(user.as_ref()).await?;
        let access = self.tokens.issue(user.as_ref())?;

        Ok(vec![
            self.cookies.issue_access(&access),
            self.cookies.issue_refresh(&refreshed),
        ])
    }

    /// Revoga o refresh e limpa os dois cookies.
    ///
    /// O access token continua tecnicamente válido até expirar — é o preço de
    /// ele ser auto-contido — mas sai do navegador aqui, e sem refresh a sessão
    /// não se renova.
    ///
    /// Revogar é **esforço, não condição**: falhar em revogar não pode impedir
    /// o cliente de sair. O que fica é o registro para quem investiga.
    async fn logout(&self, headers: HeaderMap) -> Vec<Cookie<'static>> {
        if let Some(presented) = self.cookies.read_refresh(&headers) {
            if let Err(error) = self.revoke(&presented).await {
                self.logger.warn(
                    "não foi possível revogar o refresh token no logout",
                    [("error", &error.to_string())],
                );
            }
        }

        vec![self.cookies.clear_access(), self.cookies.clear_refresh()]
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

/// Traduz a recusa do serviço de sessão no status que o cliente recebe.
///
/// Credencial errada é 401, e um sistema já montado é 409: o `POST /setup` só
/// abre uma vez na vida de um deploy, e a segunda tentativa contradiz o estado
/// em vez de estar malformada.
fn session_refused(error: SessionError) -> ApiError {
    match error {
        SessionError::InvalidCredentials => {
            ApiError::new(StatusCode::UNAUTHORIZED, error.to_string())
        }
        SessionError::AlreadySetUp => ApiError::new(StatusCode::CONFLICT, error.to_string()),
        SessionError::App(shared) => ApiError::of_app(shared),
    }
}

/// Traduz a recusa do serviço de marcação.
///
/// Ele só devolve o erro comum: marca inexistente responde `false`, não erro.
fn mark_refused(error: MarkerError) -> ApiError {
    match error {
        MarkerError::App(shared) => ApiError::of_app(shared),
    }
}
