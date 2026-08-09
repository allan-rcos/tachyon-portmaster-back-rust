//! O contrato de quem emite e lê os cookies de sessão.

use axum::http::HeaderMap;
use cookie::Cookie;

/// Emite e lê os cookies que carregam a sessão.
///
/// Trait pela mesma razão do [`TokenService`](crate::token::token_service::TokenService):
/// o controller de auth pede "o cookie que limpa o refresh" e recebe um
/// [`Cookie`], sem saber como ele é montado nem sob que nome viaja.
pub(crate) trait AuthCookie: Clone + Send + Sync + 'static {
    /// O cookie que publica o access token.
    fn issue_access(&self, token: &str) -> Cookie<'static>;

    /// O cookie que publica o refresh token.
    fn issue_refresh(&self, token: &str) -> Cookie<'static>;

    /// O cookie que apaga o access token.
    fn clear_access(&self) -> Cookie<'static>;

    /// O cookie que apaga o refresh token.
    fn clear_refresh(&self) -> Cookie<'static>;

    /// O access token apresentado, se houver.
    fn read_access(&self, headers: &HeaderMap) -> Option<String>;

    /// O refresh token apresentado, se houver.
    fn read_refresh(&self, headers: &HeaderMap) -> Option<String>;
}
