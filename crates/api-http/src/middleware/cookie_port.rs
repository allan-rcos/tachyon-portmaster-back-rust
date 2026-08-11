//! O contrato de quem lê e escreve os cookies da requisição.

use crate::ports::cookie::cookie_name::CookieName;
use crate::ports::error::api_error::ApiError;

/// Os cookies da requisição corrente.
///
/// Um controller pede "grave o access token" e "apague o refresh", e nada mais:
/// não escolhe `Path`, não escolhe `Max-Age`, não vê um `Cookie` e não decide
/// como ele viaja. Essa política é do adaptador, decidida em compilação pela
/// [`SessionPolicy`](crate::ports::session_policy::SessionPolicy).
///
/// ## Por que ela substituiu um `Vec<Cookie>` de retorno
///
/// Os quatro métodos do controller de sessão devolviam `(VO, Vec<Cookie>)`, e
/// cada rota dobrava aquele vetor sobre a resposta com um `fold`. O tipo interno
/// do crate `cookie` aparecia na assinatura de um contrato, o `ApiResponse`, o
/// `ApiError` e o `NoContent` tinham cada um o seu `with_cookie`, e emitir um
/// cookie de um ponto que não fosse o retorno de um handler era impossível.
///
/// Agora quem quer um cookie o escreve onde estiver; o layer recolhe o que foi
/// escrito e carimba na resposta.
pub(crate) trait CookiePort: Clone + Send + Sync + 'static {
    /// O valor apresentado sob este nome, se houver.
    ///
    /// Um cookie presente e vazio conta como ausente: é o que um `Max-Age=0`
    /// deixa para trás em alguns clientes, e tratá-lo como valor faria o logout
    /// parecer não ter funcionado.
    fn read(&self, name: CookieName) -> Result<Option<String>, ApiError>;

    /// Publica um valor sob este nome.
    fn set(&self, name: CookieName, value: &str) -> Result<(), ApiError>;

    /// Apaga o cookie deste nome no cliente.
    fn clear(&self, name: CookieName) -> Result<(), ApiError>;
}
