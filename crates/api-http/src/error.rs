//! O erro como o cliente o vê.
//!
//! Esta é a **única** camada que conhece status HTTP. O `domain` devolve erro
//! tipado, o `app` o une num [`AppError`], e a tradução para número acontece
//! aqui — em [`app_error_to_status`], num lugar só. Espalhar o código pelo
//! sistema foi o que o PHP fazia (o `LeafContext` carregava o status desde o
//! domínio), e o preço era que a mesma violação de regra tinha um status gravado
//! nela mesmo quando a saída não era HTTP.
//!
//! ## O corpo é sempre `application/problem+json`
//!
//! Mesmo quando a requisição pediu FlatBuffers. É o que o PHP fazia, e há uma
//! razão para manter: um erro pode acontecer **antes** de a negociação ter sido
//! resolvida — corpo malformado, `Accept` ilegível, pânico — e nesses casos não
//! existe formato negociado para responder. Ter um formato de erro único e
//! sempre disponível evita a pergunta "em que formato eu digo que não entendi o
//! formato?".

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use portmaster_app::domain::FieldError;
use portmaster_app::error::AppError;
use serde::Serialize;

/// O tipo de mídia do corpo de erro (RFC 7807).
const PROBLEM_JSON: &str = "application/problem+json";

/// Um erro pronto para virar resposta.
///
/// Carrega cookies porque uma recusa às vezes **precisa** mexer na sessão: um
/// refresh token morto tem que sair do navegador junto com o 401, senão o
/// cliente o reapresenta a cada tentativa e nunca chega ao login. É o único caso,
/// e é da mesma natureza do status — parte da resposta, não do erro.
#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    detail: String,
    cookies: Vec<String>,
}

/// O corpo, na forma da RFC 7807 — espelha `ProblemDetails` de `common.fbs`.
#[derive(Debug, Serialize)]
struct ProblemDetails<'a> {
    /// URI do tipo de problema; `about:blank` quando não há uma página.
    r#type: &'a str,
    /// O nome canônico do status.
    title: &'a str,
    /// O status, repetido no corpo para quem só lê o payload.
    status: u16,
    /// O que aconteceu, em texto.
    detail: &'a str,
}

impl ApiError {
    /// Um erro com status e explicação.
    pub(crate) fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
            cookies: Vec::new(),
        }
    }

    /// Acrescenta um `Set-Cookie` à recusa.
    pub(crate) fn with_cookie(mut self, cookie: String) -> Self {
        self.cookies.push(cookie);
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
    pub(crate) fn malformed_body(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }

    /// O status, para quem precisa decidir sobre ele antes de responder.
    ///
    /// Só os testes perguntam: no caminho de produção o erro vira resposta
    /// direto, sem ninguém inspecioná-lo pelo meio.
    #[cfg(test)]
    pub(crate) fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let problem = ProblemDetails {
            r#type: "about:blank",
            title: title_of(self.status),
            status: self.status.as_u16(),
            detail: &self.detail,
        };

        // A serialização de uma struct de quatro campos escalares não falha; se
        // um dia falhasse, um corpo vazio com o status certo ainda é uma
        // resposta melhor do que um pânico dentro do handler de erro.
        let body = serde_json::to_vec(&problem).unwrap_or_default();

        let mut response =
            (self.status, [(header::CONTENT_TYPE, PROBLEM_JSON)], body).into_response();

        for cookie in self.cookies {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }

        response
    }
}

/// Traduz o erro do `app` para o status que o cliente recebe.
///
/// É o **único** ponto de tradução do sistema. Um `match` exaustivo garante que
/// uma variante nova de [`AppError`] não passe despercebida como 500.
pub(crate) fn app_error_to_status(error: AppError) -> ApiError {
    match error {
        // Validação vem em lote — o domínio acumula todos os campos em vez de
        // parar no primeiro. Juntá-los num `detail` só é o que o
        // `ProblemDetails` do schema comporta: ele não tem `details[]`, e o
        // `.fbs` é contrato com o cliente, não algo a alterar por conveniência.
        AppError::Validation(fields) => {
            ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, describe_fields(&fields))
        }

        AppError::Unauthenticated => ApiError::new(
            StatusCode::UNAUTHORIZED,
            AppError::Unauthenticated.to_string(),
        ),

        // O slug **não** vai no corpo. Dizer ao cliente qual permissão faltou
        // descreve para ele o mapa de autorização do sistema; quem precisa do
        // detalhe é o operador, e para esse ele já está no log.
        AppError::Forbidden { permission } => {
            tracing::info!(permission, "acesso negado por falta de permissão");
            ApiError::new(
                StatusCode::FORBIDDEN,
                "You do not have permission to perform this action.",
            )
        }

        AppError::NotFound { resource, id } => ApiError::new(
            StatusCode::NOT_FOUND,
            format!("{resource} não encontrado: {id}"),
        ),

        AppError::Conflict(message) => ApiError::new(StatusCode::CONFLICT, message),

        // Regras de negócio violadas são conflito de estado, não erro de
        // formato: o pedido estava bem escrito, e é o contêiner que não podia
        // ser selado agora.
        AppError::Container(e) => ApiError::new(StatusCode::CONFLICT, e.to_string()),
        AppError::Manifest(e) => ApiError::new(StatusCode::CONFLICT, e.to_string()),
        AppError::Marker(e) => ApiError::new(StatusCode::CONFLICT, e.to_string()),
        AppError::Metadata(e) => ApiError::new(StatusCode::CONFLICT, e.to_string()),

        // Credencial recusada. A mensagem é a mesma para e-mail desconhecido e
        // senha errada — o `app` já garante isso.
        AppError::Auth(e) => ApiError::new(StatusCode::UNAUTHORIZED, e.to_string()),

        // O que a infra reportou fica no log com a cadeia inteira; o cliente
        // recebe só que houve falha. O motivo real ("a conexão com o banco foi
        // recusada") não é acionável para ele e descreve a topologia interna.
        AppError::Infra(e) => {
            tracing::error!(error = ?e, "falha de infraestrutura");
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred.",
            )
        }
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

/// O nome canônico de um status.
fn title_of(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("Error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_permissao_negada_nao_vaza_o_slug_no_corpo() {
        // O slug vai para o log. No corpo, ele descreveria ao cliente o mapa de
        // autorização do sistema.
        let error = app_error_to_status(AppError::Forbidden {
            permission: "container:seal".into(),
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
        let error = app_error_to_status(AppError::Infra(anyhow::anyhow!(
            "Connection refused (os error 111) para db:3306"
        )));

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            !error.detail.contains("db:3306"),
            "topologia vazou: {}",
            error.detail
        );
    }

    #[test]
    fn a_validacao_lista_todos_os_campos() {
        // O domínio acumula; perder campos aqui obrigaria o cliente a descobrir
        // um problema por requisição.
        let error = app_error_to_status(AppError::Validation(vec![
            FieldError::new("email", "malformado"),
            FieldError::new("password", "curta demais"),
        ]));

        assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(error.detail.contains("email"));
        assert!(error.detail.contains("password"));
    }

    #[test]
    fn a_regra_de_negocio_violada_e_conflito() {
        // O pedido estava bem escrito; o que não podia era o estado do contêiner.
        use portmaster_app::domain::ContainerStatus;
        let _ = ContainerStatus::Empty;

        let error = app_error_to_status(AppError::Conflict(
            "Only a sealed container can be dispatched.".into(),
        ));

        assert_eq!(error.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn o_corpo_de_erro_e_problem_json() {
        let response = ApiError::unauthenticated().into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(PROBLEM_JSON)
        );
    }
}
