//! O controller de sessão. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::marker::{RegisterMarkerGroupCommand, SetMarkerCommand};
use portmaster_app::commands::session::{LoginCommand, SetupCommand};
use portmaster_app::context::UserContext;
use portmaster_app::domain::User;
use portmaster_app::error::{MarkerError, SessionError};
use portmaster_app::queries::marker::GetMarkerQuery;
use portmaster_app::services::{MarkService, SessionService};
use portmaster_app::{Logger, RandomIdGenerator};

use crate::controllers::auth_controller::AuthController;
use crate::middleware::cookie_port::CookiePort;
use crate::ports::cookie::cookie_name::CookieName;
use crate::ports::error::api_error::ApiError;
use crate::ports::token::refresh_token::RefreshToken;
use crate::ports::token::token_service::TokenService;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
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

/// Monta o controller de sessão.
///
/// Sete dependências, e é o controller mais costurado do sistema: ele emite
/// duas coisas com prazos diferentes — o access token e o refresh —, escreve os
/// dois em cookie e ainda registra o grupo de marcador que guarda o refresh.
pub(crate) fn auth_controller<S, M, R, T, A, L>(
    sessions: S,
    marks: M,
    random: R,
    tokens: T,
    cookies: A,
    logger: L,
    refresh_ttl_seconds: u64,
) -> impl AuthController + use<S, M, R, T, A, L> + 'static
where
    S: SessionService + Clone + Send + Sync + 'static,
    M: MarkService + Clone + Send + Sync + 'static,
    R: RandomIdGenerator + Clone + Send + Sync + 'static,
    T: TokenService + Clone + Send + Sync + 'static,
    A: CookiePort + Clone + Send + Sync + 'static,
    L: Logger + Clone + Send + Sync + 'static,
{
    AuthControllerImpl {
        sessions,
        marks,
        random,
        tokens,
        cookies,
        logger,
        refresh_ttl_seconds,
    }
}

/// Os handlers de sessão, genéricos sobre tudo que consomem.
///
/// Repare que `T`, `A` e `L` são parâmetros de tipo e não structs concretas:
/// declarar os campos com as impls faria a hierarquia que as traits desenham
/// valer para fora do crate mas não para dentro dele.
///
/// É o único controller que injeta a [`CookiePort`], e é a razão de ela existir:
/// a sessão **é** um par de cookies, e quem decide o que entra neles é quem
/// emite o token. O que a porta garante é que ele o faça sem ver um `Cookie`,
/// sem escolher `Path` nem `Max-Age`, e sem que o tipo interno do crate `cookie`
/// apareça na assinatura de contrato nenhum.
#[derive(Clone)]
struct AuthControllerImpl<S, M, R, T, A, L> {
    /// O service de sessão.
    sessions: S,
    /// O service de marcador, que guarda o refresh.
    marks: M,
    /// De onde sai a metade aleatória do refresh token.
    random: R,
    /// Quem emite e confere o access token.
    tokens: T,
    /// Por onde os cookies de sessão são escritos e lidos.
    cookies: A,
    /// Para onde vai o que não impediu a operação.
    logger: L,
    /// Por quanto tempo o refresh vale, em segundos.
    refresh_ttl_seconds: u64,
}

impl<S, M, R, T, A, L> AuthControllerImpl<S, M, R, T, A, L>
where
    S: SessionService,
    M: MarkService,
    R: RandomIdGenerator,
    T: TokenService,
    A: CookiePort,
    L: Logger,
{
    /// Emite access e refresh, publica os cookies e monta o corpo da sessão.
    async fn issue_session(&self, user: &dyn User) -> Result<LoginXResponse, ApiError> {
        let refresh = self.mint_refresh(user).await?;
        let access = self.tokens.issue(user)?;

        self.cookies.set(CookieName::Access, &access)?;
        self.cookies.set(CookieName::Refresh, &refresh)?;

        Ok(LoginXResponse {
            token: access,
            token_type: TOKEN_TYPE.to_owned(),
            user: UserX::of(user),
        })
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
    S: SessionService + Clone + Send + Sync + 'static,
    M: MarkService + Clone + Send + Sync + 'static,
    R: RandomIdGenerator + Clone + Send + Sync + 'static,
    T: TokenService,
    A: CookiePort,
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
    async fn setup(self, Body(request): Body<SetupXRequest>) -> ApiResponse<LoginXResponse> {
        ApiResponse::created(
            async {
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
            .await,
        )
    }

    /// Abre a sessão.
    ///
    /// Credencial ausente vira string vazia e falha como credencial **errada**:
    /// o `app` responde igual para e-mail desconhecido e senha errada, e
    /// distinguir "não mandou o campo" aqui entregaria ao atacante metade da
    /// resposta.
    async fn login(self, Body(request): Body<LoginXRequest>) -> ApiResponse<LoginXResponse> {
        ApiResponse::ok(
            async {
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
            .await,
        )
    }

    /// Troca um refresh válido por um par novo.
    ///
    /// Sem cookie não há o que limpar, e limpar assim mesmo mandaria um
    /// `Set-Cookie` para quem nunca teve sessão. Já um token apresentado e morto
    /// **é** tirado do navegador — é o que impede o cliente de reapresentá-lo
    /// para sempre.
    async fn refresh(self) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let Some(presented) = self.cookies.read(CookieName::Refresh)? else {
                    return Err(refused());
                };

                let user = match self.rotate(&presented).await {
                    Ok(user) => user,
                    Err(error) => {
                        self.cookies.clear(CookieName::Access)?;
                        self.cookies.clear(CookieName::Refresh)?;

                        return Err(error);
                    }
                };

                let refreshed = self.mint_refresh(user.as_ref()).await?;
                let access = self.tokens.issue(user.as_ref())?;

                self.cookies.set(CookieName::Access, &access)?;
                self.cookies.set(CookieName::Refresh, &refreshed)?;

                Ok(())
            }
            .await,
        )
    }

    /// Revoga o refresh e limpa os dois cookies.
    ///
    /// O access token continua tecnicamente válido até expirar — é o preço de
    /// ele ser auto-contido — mas sai do navegador aqui, e sem refresh a sessão
    /// não se renova.
    ///
    /// Revogar é **esforço, não condição**: falhar em revogar não pode impedir
    /// o cliente de sair. O que fica é o registro para quem investiga.
    async fn logout(self) -> ApiResponse {
        ApiResponse::no_content(
            async {
                if let Ok(Some(presented)) = self.cookies.read(CookieName::Refresh) {
                    if let Err(error) = self.revoke(&presented).await {
                        self.logger.warn(
                            "não foi possível revogar o refresh token no logout",
                            [("error", &error.to_string())],
                        );
                    }
                }

                self.cookies.clear(CookieName::Access)?;
                self.cookies.clear(CookieName::Refresh)?;

                Ok(())
            }
            .await,
        )
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
