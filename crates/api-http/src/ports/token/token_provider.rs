//! Quem serve o emissor de token.

use std::sync::{PoisonError, RwLock};

use anyhow::Context as _;

use crate::config::jwt_config::JwtConfig;
use crate::ports::token::adapter::jwt_token_service::{jwt_token_service, JwtTokenService};
use crate::ports::token::token_service::TokenService;

/// O emissor de token do processo.
///
/// Guardado porque tem estado: as duas chaves HS256 já derivadas do segredo, e
/// dois consumidores as querem — o controller de sessão e o `SessionLayer`.
///
/// `RwLock` porque quem o cria é configuração, e configuração se troca.
static TOKEN_SERVICE: RwLock<Option<JwtTokenService>> = RwLock::new(None);

/// Quem emite e confere o access token.
pub(crate) struct TokenProvider;

impl TokenProvider {
    /// Deriva as chaves deste segredo, e larga as que estavam em uso.
    ///
    /// É rotação de chave: todo token já emitido deixa de ser aceito, porque a
    /// chave que o assinou não é mais a que confere a assinatura. O
    /// `SessionLayer` já montado é a exceção — ele segura um clone do emissor
    /// antigo e segue aceitando os tokens dele até o router ser remontado.
    pub(crate) fn install(config: &JwtConfig) {
        *TOKEN_SERVICE
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Some(jwt_token_service(config));
    }

    /// O emissor de token do processo.
    ///
    /// Sem segredo instalado, isto falha em vez de inventar um padrão. O padrão
    /// de [`JwtConfig`] tem o segredo vazio, e um HS256 com segredo vazio
    /// aceita token que qualquer um forja — é exatamente o desfecho que o elo
    /// de config recusa no boot, e não faria sentido reabri-lo aqui.
    pub(crate) fn token_service() -> anyhow::Result<impl TokenService + use<> + 'static> {
        TOKEN_SERVICE
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .context("o emissor de token ainda não foi criado: o segredo do JWT precisa ser instalado antes")
    }
}
