//! A falha que qualquer caso de uso pode devolver.

use portmaster_domain::error::FieldError;

use crate::error::AppErrorKind;

/// O que pode dar errado em **qualquer** caso de uso.
///
/// São as três recusas que não pertencem a serviço nenhum: o corpo que não passa
/// pela forma, a permissão que falta e a infraestrutura que caiu. Tudo o mais —
/// "produto não encontrado", "esse contêiner não está selado" — é do serviço que
/// levanta, e mora no erro dele.
///
/// As variantes descrevem a falha no vocabulário do sistema, não no do protocolo
/// por onde ela sairá. Quem precisa decidir em cima do erro sem casar variante
/// por variante usa [`AppError::kind`]; quem precisa de um status HTTP é o
/// `api-http`, e a tradução mora lá.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Campos inválidos, em lote.
    ///
    /// Lista e não primeiro-erro: quem preencheu um formulário quer corrigir
    /// tudo de uma vez, não descobrir um problema por requisição.
    #[error("dados inválidos")]
    Validation(Vec<FieldError>),

    /// Falta a permissão exigida.
    ///
    /// O slug viaja para o log. O `api-http` **não** o coloca no corpo — é o
    /// comportamento que o PHP tinha, e dizer ao cliente qual permissão faltou
    /// descreve para ele o mapa de autorização do sistema.
    #[error("permissão negada: {permission}")]
    PermissionDenied {
        /// O slug que faltava.
        permission: &'static str,
    },

    /// Falha de I/O ou de infraestrutura.
    ///
    /// Opaca de propósito: "o banco recusou a conexão" não é uma regra que o
    /// cliente possa satisfazer, e detalhar o motivo só vazaria topologia.
    #[error(transparent)]
    Infra(#[from] anyhow::Error),
}

impl AppError {
    /// A recusa de quem não tem a permissão exigida.
    ///
    /// O slug é `&'static str` e não `String`: ele é uma constante do binário,
    /// declarada pelo serviço que a exige, e alocá-la a cada recusa seria
    /// trabalho por nada.
    pub const fn permission_denied(permission: &'static str) -> Self {
        Self::PermissionDenied { permission }
    }

    /// De que natureza é esta falha.
    ///
    /// O agrupamento é o contrato estável desta camada; quem decide em cima do
    /// erro não precisa casar variante por variante.
    pub const fn kind(&self) -> AppErrorKind {
        match *self {
            Self::Validation(_) => AppErrorKind::Validation,
            Self::PermissionDenied { .. } => AppErrorKind::Authorization,
            Self::Infra(_) => AppErrorKind::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_slug_negado_fica_no_erro() {
        // Vai para o log; quem decide não mostrá-lo ao cliente é o api-http.
        let error = AppError::permission_denied("product:create");

        assert!(error.to_string().contains("product:create"));
    }
}
