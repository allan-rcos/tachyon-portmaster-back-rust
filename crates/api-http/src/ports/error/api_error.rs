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
//! pelo [`Encoder`] da requisição — em JSON ou em `FlatBuffers`, conforme o
//! `Accept`. Antes o corpo era `application/problem+json` fixo, o que num
//! sistema cujo cliente de produção fala `FlatBuffers` significava que todo
//! caminho de erro entregava algo que ele não sabia ler.
//!
//! Quando não há requisição de onde negociar — um erro que nasce antes de
//! qualquer cabeçalho ser lido — vale o padrão do [`Encoder`], que é JSON. É a
//! mesma escolha que um `Accept` irreconhecível recebe.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cookie::Cookie;
use portmaster_app::domain::FieldError;
use portmaster_app::error::{AppError, AppErrorKind};
use portmaster_app::{Logger as _, SystemLogger};

use crate::wire::encoder::Encoder;
use crate::wire::vo::common::problem_x::ProblemX;

/// Um erro pronto para virar resposta.
///
/// Carrega cookies porque uma recusa às vezes **precisa** mexer na sessão: um
/// refresh token morto tem que sair do navegador junto com o 401, senão o
/// cliente o reapresenta a cada tentativa e nunca chega ao login. É o único caso,
/// e é da mesma natureza do status — parte da resposta, não do erro.
#[derive(Debug)]
pub struct ApiError {
    /// O status HTTP da resposta.
    status: StatusCode,
    /// O que aconteceu, em texto, para o corpo do problema.
    detail: String,
    /// Os `Set-Cookie` a acrescentar na resposta, um cabeçalho por entrada.
    cookies: Vec<Cookie<'static>>,
    /// Como escrever o corpo, quando este erro virar resposta sozinho.
    ///
    /// Só importa para a recusa de um extractor, que vira resposta sem passar
    /// por um [`ApiResponse`](crate::wire::api_response::ApiResponse). Nos
    /// demais caminhos quem codifica é o encoder da resposta.
    encoder: Encoder,
}

impl ApiError {
    /// Um erro com status e explicação.
    pub(crate) fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
            cookies: Vec::new(),
            encoder: Encoder::default(),
        }
    }

    /// Acrescenta um `Set-Cookie` à recusa.
    #[must_use]
    pub(crate) fn with_cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }

    /// Fixa por onde este erro sai, se sair sozinho.
    #[must_use]
    pub(crate) const fn with_encoder(mut self, encoder: Encoder) -> Self {
        self.encoder = encoder;
        self
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

    /// O status, para quem precisa decidir sobre ele antes de responder.
    ///
    /// Só os testes perguntam: no caminho de produção o erro vira resposta
    /// direto, sem ninguém inspecioná-lo pelo meio.
    #[cfg(test)]
    pub(crate) const fn status(&self) -> StatusCode {
        self.status
    }

    /// Desmonta o erro no que a resposta precisa.
    ///
    /// Devolver as três peças de uma vez é o que permite ao
    /// [`ApiResponse`](crate::wire::api_response::ApiResponse) codificar o
    /// problema pelo **seu** encoder, e não pelo que este erro carrega — que é
    /// só o de reserva, para quando ele vira resposta sozinho.
    pub(crate) fn into_parts(self) -> (StatusCode, ProblemX, Vec<Cookie<'static>>) {
        let problem = ProblemX {
            kind: "about:blank",
            title: self.status.canonical_reason().unwrap_or("Error").to_owned(),
            status: i32::from(self.status.as_u16()),
            detail: self.detail,
        };

        (self.status, problem, self.cookies)
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
    /// Codifica o problema pelo encoder de reserva.
    ///
    /// Este caminho é o da recusa de extractor — o corpo que não deu para ler, a
    /// sessão que falta. Quem o alcança já anexou o encoder da requisição com
    /// [`Self::with_encoder`] quando tinha um.
    fn into_response(self) -> Response {
        let encoder = self.encoder;
        let (status, problem, cookies) = self.into_parts();

        encoder.respond(status, &problem, cookies)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use pretty_assertions::assert_eq;

    /// O slug vai para o log.
    ///
    /// No corpo, ele descreveria ao cliente o mapa de autorização do sistema.
    #[test]
    fn a_permissao_negada_nao_vaza_o_slug_no_corpo() {
        let error = ApiError::of_app(AppError::PermissionDenied {
            permission: "container:seal",
        });

        assert_eq!(error.status(), StatusCode::FORBIDDEN);
        assert!(
            !error.detail.contains("container:seal"),
            "o corpo não deveria nomear a permissão: {}",
            error.detail
        );
    }

    #[test]
    fn a_falha_de_infra_nao_vaza_o_motivo() {
        let error = ApiError::of_app(AppError::Infra(anyhow::anyhow!(
            "Connection refused (os error 111) para db:3306"
        )));

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            !error.detail.contains("db:3306"),
            "topologia vazou: {}",
            error.detail
        );
    }

    /// O domínio acumula; perder campos aqui obrigaria o cliente a descobrir
    /// um problema por requisição.
    #[test]
    fn a_validacao_lista_todos_os_campos() {
        let error = ApiError::of_app(AppError::Validation(vec![
            FieldError::new("email", "malformado"),
            FieldError::new("password", "curta demais"),
        ]));

        assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(error.detail.contains("email"));
        assert!(error.detail.contains("password"));
    }

    /// O slug negado não aparece no corpo, mas o 403 aparece.
    #[test]
    fn a_permissao_que_falta_e_proibido() {
        let error = ApiError::of_app(AppError::permission_denied("container:dispatch"));

        assert_eq!(error.status(), StatusCode::FORBIDDEN);
        assert!(
            !error.detail.contains("container:dispatch"),
            "o slug descreve o mapa de autorização para quem acabou de ser recusado"
        );
    }

    /// Sem requisição de onde negociar, vale o padrão — e o padrão é JSON.
    #[test]
    fn o_erro_sem_negociacao_sai_em_json() {
        let response = ApiError::unauthenticated().into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }

    /// É o item que motivou o desenho: um cliente que fala `FlatBuffers` recebe
    /// o erro em `FlatBuffers`, e não num JSON que ele não sabe ler.
    #[test]
    fn o_erro_sai_no_formato_que_o_cliente_pediu() {
        let response = ApiError::unauthenticated()
            .with_encoder(Encoder::of_response(Some("application/x-flatbuffers")))
            .into_response();

        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/x-flatbuffers")
        );
    }
}
