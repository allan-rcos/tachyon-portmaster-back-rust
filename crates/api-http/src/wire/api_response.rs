//! A resposta de um controller, já negociada.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::middleware::encode_port::EncodePort as _;
use crate::middleware::intern::encode_context::EncodeContext;
use crate::ports::error::api_error::ApiError;
use crate::wire::vo::common::problem_x::ProblemX;
use crate::wire::x::response_x::ResponseX;

/// O que um controller produziu, pronto para virar resposta.
///
/// Genérica sobre o VO. Não há `Box<dyn>` aqui, e não há como haver: o corpo é
/// um tipo concreto que a rota conhece em tempo de compilação, e a codificação é
/// monomorfizada junto com ele.
///
/// ## Por que ela carrega um `Result`
///
/// Porque o erro precisa sair pelo mesmo formato que o acerto. Se o sucesso
/// passasse por aqui e a falha por outro caminho, existiriam dois lugares
/// escrevendo corpo — e um deles acabaria escrevendo num formato que o cliente
/// não pediu. Envolvendo o `Result`, negociar é uma coisa só que acontece uma
/// vez, no [`IntoResponse`] abaixo.
///
/// ## O parâmetro tem padrão
///
/// Porque uma resposta sem corpo não tem VO nenhum, e `ApiResponse::no_content()`
/// precisa resolver sozinho. `ProblemX` é o que ela usaria de qualquer forma se
/// o `Result` desse errado — o corpo de erro é o mesmo em toda resposta.
///
/// ## E por que o corpo é `Option`
///
/// Porque `204` é uma resposta como as outras. Era um tipo à parte — o
/// `NoContent` —, o que dava duas formas de responder. Um controller que às
/// vezes tem corpo e às vezes não tinha de escolher entre elas no meio do
/// método.
pub(crate) struct ApiResponse<X: ResponseX = ProblemX> {
    /// O status do acerto — o da falha vem do próprio erro.
    status: StatusCode,
    /// O que responder, ou por que não dá; `Ok(None)` é o `204`.
    body: Result<Option<X>, ApiError>,
}

impl<X: ResponseX> ApiResponse<X> {
    /// Um `200` com o que o controller devolveu.
    pub(crate) fn ok(body: Result<X, ApiError>) -> Self {
        Self::with_status(StatusCode::OK, body)
    }

    /// Um `201` para o recurso recém-criado.
    pub(crate) fn created(body: Result<X, ApiError>) -> Self {
        Self::with_status(StatusCode::CREATED, body)
    }

    /// A resposta com um status escolhido.
    pub(crate) fn with_status(status: StatusCode, body: Result<X, ApiError>) -> Self {
        Self {
            status,
            body: body.map(Some),
        }
    }
}

impl ApiResponse<ProblemX> {
    /// Um `204` para a operação cujo resultado é o próprio estado ter mudado.
    ///
    /// É o que o PHP devolvia em refresh, logout e nas operações sem retorno.
    /// Recebe um `Result` como as outras duas, e pela mesma razão: a falha
    /// precisa sair pelo mesmo caminho do acerto, com o status que o erro
    /// carrega.
    ///
    /// Fixada em `ProblemX` porque uma resposta sem corpo não tem VO nenhum, e o
    /// parâmetro precisa de algum: o `ProblemX` é o que ela usaria de qualquer
    /// forma se o `Result` desse errado. Fixá-la aqui é também o que a faz
    /// resolver sem turbofish.
    pub(crate) fn no_content(body: Result<(), ApiError>) -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            body: body.map(|()| None),
        }
    }
}

impl<X: ResponseX> IntoResponse for ApiResponse<X> {
    /// Codifica o corpo — ou o problema — no formato negociado.
    ///
    /// Cookie não aparece aqui. Quem os carimba é o layer de cookie, depois do
    /// handler e para toda resposta, o que é o que faz uma recusa poder mexer na
    /// sessão sem que este tipo saiba disso.
    fn into_response(self) -> Response {
        match self.body {
            Ok(Some(body)) => EncodeContext.respond(self.status, &body),
            Ok(None) => self.status.into_response(),
            Err(error) => {
                let (status, problem) = error.into_parts();

                EncodeContext.respond(status, &problem)
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/api_response_test.rs"]
mod tests;
