//! O erro como o cliente o vê.
//!
//! Esta é a **única** camada que conhece status HTTP. O `domain` devolve erro
//! tipado, o `app` o une num [`AppError`] e o agrupa por natureza, e a tradução
//! para número acontece aqui — em [`ApiError::of_app`], num lugar só. Espalhar o
//! código pelo sistema foi o que o PHP fazia (o `LeafContext` carregava o status
//! desde o domínio), e o preço era que a mesma violação de regra tinha um status
//! gravado nela mesmo quando a saída não era HTTP.
//!
//! ## O corpo de erro é negociado como qualquer outro
//!
//! Um erro vira [`ProblemX`], que é um VO de resposta como qualquer outro, e sai
//! pela [`EncodePort`](crate::middleware::encode_port::EncodePort) da
//! requisição — em JSON ou em `FlatBuffers`, conforme o `Accept`. Um corpo fixo
//! em `application/problem+json` faria, num sistema cujo cliente de produção
//! fala `FlatBuffers`, todo caminho de erro entregar algo que ele não sabe ler.
//!
//! Quando não há requisição de onde negociar — um erro que nasce antes de
//! qualquer cabeçalho ser lido — vale JSON, que é o padrão do contexto fora do
//! escopo.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use portmaster_app::domain::FieldError;
use portmaster_app::error::{AppError, AppErrorKind};
use portmaster_app::{Logger as _, SystemLogger};

use crate::middleware::encode_port::EncodePort as _;
use crate::middleware::intern::encode_context::EncodeContext;
use crate::wire::vo::common::problem_x::ProblemX;

/// Um erro pronto para virar resposta.
///
/// Um status e uma explicação, e nada mais. Ele carregava cookies, porque uma
/// recusa às vezes precisa mexer na sessão — um refresh token morto tem que sair
/// do navegador junto com o `401`, senão o cliente o reapresenta a cada
/// tentativa e nunca chega ao login. Isso continua acontecendo, só que pela
/// [`CookiePort`](crate::middleware::cookie_port::CookiePort): quem descobriu
/// que o token morreu escreve o cookie ali, e o layer o carimba na resposta
/// qualquer que ela seja.
#[derive(Debug)]
pub struct ApiError {
    /// O status HTTP da resposta.
    status: StatusCode,
    /// O que aconteceu, em texto, para o corpo do problema.
    detail: String,
}

impl ApiError {
    /// Um erro com status e explicação.
    pub(crate) fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    /// Rota protegida sem sessão.
    ///
    /// O **401 é o único status que nasce nesta camada**: é a ausência de
    /// sessão, e só quem lê o token sabe disso. Permissão (403), validação (422)
    /// e conflito (409) vêm todos do `app`.
    pub(crate) fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "Authentication is required to access this resource.",
        )
    }

    /// Corpo que não deu para ler no formato anunciado.
    ///
    /// **400, e não o 404 que o PHP passou a devolver.** O 404 é afirmado pela
    /// suíte Go em cinco pontos para recurso genuinamente ausente; colidir os
    /// dois significaria que nem um painel nem uma política de retry conseguem
    /// separar "essa rota não existe" de "seus bytes são lixo".
    ///
    /// Cobre também o corpo **vazio**: ausência é um caso de ilegibilidade, não
    /// um caso à parte.
    pub(crate) fn unreadable_body(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }

    /// Resposta que existe mas não coube no fio.
    ///
    /// 502 e não 500: distingue "a resposta foi montada e falhou ao serializar"
    /// do 500 que o middleware `Recover` produz quando algo entrou em pânico. Os
    /// dois são defeito nosso, mas levam a investigações diferentes — e a suíte
    /// Go não afirma 502 em ponto nenhum, então o código está livre.
    pub(crate) fn unrenderable(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, detail)
    }

    /// Desmonta o erro no status e no corpo de problema.
    pub(crate) fn into_parts(self) -> (StatusCode, ProblemX) {
        let problem = ProblemX {
            kind: "about:blank",
            title: self.status.canonical_reason().unwrap_or("Error").to_owned(),
            status: i32::from(self.status.as_u16()),
            detail: self.detail,
        };

        (self.status, problem)
    }

    /// Traduz o erro **comum** do `app` para o status que o cliente recebe.
    ///
    /// São as três recusas que qualquer caso de uso pode devolver, e a tradução
    /// delas mora num lugar só. As recusas próprias de um serviço — "produto não
    /// encontrado", "esse contêiner não está selado" — são casadas pelo
    /// controller que chamou aquele caso de uso, que é quem sabe o que cada uma
    /// significa na rota dele.
    ///
    /// ## Validação vem em lote
    ///
    /// O domínio acumula **todos** os campos em vez de parar no primeiro.
    /// Juntá-los num `detail` só é o que o `ProblemDetails` do schema comporta:
    /// ele não tem `details[]`, e o `.fbs` é contrato com o cliente, não algo a
    /// alterar por conveniência.
    ///
    /// ## O slug da permissão negada não vai no corpo
    ///
    /// Dizer ao cliente qual permissão faltou descreve para ele o mapa de
    /// autorização do sistema. Quem precisa do detalhe é o operador, e para esse
    /// ele já está no log.
    ///
    /// ## Falha de infra não descreve a topologia
    ///
    /// O que a infra reportou fica no log com a cadeia inteira; o cliente recebe
    /// só que houve falha. O motivo real ("a conexão com o banco foi recusada")
    /// não é acionável para ele.
    pub fn of_app(error: AppError) -> Self {
        let status = match error.kind() {
            AppErrorKind::Validation => StatusCode::UNPROCESSABLE_ENTITY,
            AppErrorKind::Authorization => StatusCode::FORBIDDEN,
            AppErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let detail = match error {
            AppError::Validation(fields) => describe_fields(&fields),

            AppError::PermissionDenied { permission } => {
                SystemLogger::get().info(
                    "acesso negado por falta de permissão",
                    [("permission", permission)],
                );

                "You do not have permission to perform this action.".to_owned()
            }

            AppError::Infra(cause) => {
                SystemLogger::get().error(
                    "falha de infraestrutura",
                    [("error", &format!("{cause:?}"))],
                );

                "An unexpected error occurred.".to_owned()
            }
        };

        Self::new(status, detail)
    }
}

impl IntoResponse for ApiError {
    /// Codifica o problema no formato que a requisição negociou.
    ///
    /// Este caminho é o da recusa que vira resposta **sozinha** — a de um
    /// extractor, ou a de um middleware. O erro não carrega mais um encoder de
    /// reserva anexado à mão: o formato está no escopo da requisição, e a
    /// [`EncodePort`](crate::middleware::encode_port::EncodePort) o alcança de
    /// onde quer que este erro tenha nascido.
    ///
    /// É o único ponto do sistema que constrói o adaptador em vez de recebê-lo
    /// injetado, e não há alternativa: `IntoResponse::into_response` não recebe
    /// argumento nenhum. O adaptador é um ZST, então construí-lo não é nada.
    fn into_response(self) -> Response {
        let (status, problem) = self.into_parts();

        EncodeContext.respond(status, &problem)
    }
}

/// Resume os campos recusados numa linha.
fn describe_fields(fields: &[FieldError]) -> String {
    if fields.is_empty() {
        return "dados inválidos".to_owned();
    }

    fields
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
