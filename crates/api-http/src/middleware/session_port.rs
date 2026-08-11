//! O contrato de quem quer saber quem está falando.

use portmaster_app::context::UserContext;

use crate::ports::error::api_error::ApiError;

/// A sessão da requisição corrente.
///
/// Leitura e nada mais. Quem **põe** um usuário na sessão é o layer que confere
/// o token, e ele alcança o escritor por dentro do módulo — de fora não há como
/// um controller declarar-se autenticado, que é a única forma de o `401` valer
/// alguma coisa.
///
/// ## O gate está nos getters, não nos middlewares
///
/// [`Self::current_user`] falha se o middleware de token não rodou antes. Parece
/// redundante — se o middleware está no router, ele rodou — mas é o que
/// transforma um erro de ordenação da pilha em falha imediata e nomeada, em vez
/// de um `None` silencioso que o controller leria como "não há sessão" e
/// responderia `401` para todo mundo.
///
/// A diferença aparece no dia em que alguém reordena os `.layer()`: com o gate,
/// a primeira requisição explica o problema; sem ele, a API simplesmente para de
/// autenticar e ninguém sabe por quê.
pub(crate) trait SessionPort: Clone + Send + Sync + 'static {
    /// O usuário da sessão, ou `None` em rota pública.
    ///
    /// Falha — e não devolve `None` — quando o middleware de token não rodou.
    fn current_user(&self) -> Result<Option<UserContext>, ApiError>;

    /// O usuário da sessão, ou `401`.
    ///
    /// O atalho que todo handler protegido usa. O **`401` é o único status que
    /// nasce nesta camada**: falta de sessão é a única coisa que o `app` não tem
    /// como saber.
    fn require_user(&self) -> Result<UserContext, ApiError>;
}
