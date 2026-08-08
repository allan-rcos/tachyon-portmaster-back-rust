//! O contrato de quem lê um corpo de requisição.

use serde_json::{Map, Value};

use crate::error::api_error::ApiError;

/// Monta o DTO de uma requisição a partir de qualquer um dos dois formatos.
///
/// **Não tem `self`.** Uma factory não carrega estado: ela é um namespace
/// tipado, um ZST que existe só para dar nome ao par de leituras de uma
/// mensagem. É o que permite pedi-la por parâmetro de tipo — `Body<LoginRequestFactory>`
/// — em vez de instanciá-la.
///
/// ## Por que dois métodos e não um `Deserialize`
///
/// O caminho JSON recebe um `Map` já parseado e **escolhe** os campos, em vez de
/// deixar o serde derivar a struct inteira. A diferença aparece num campo
/// obrigatório ausente: com `Deserialize`, o serde falha antes de o handler
/// existir e a resposta é um 400 genérico; escolhendo campo a campo, o DTO
/// recebe `None`, o `TableModule` recusa e o cliente ganha um 422 dizendo
/// **qual** campo faltou — e todos os que faltaram, de uma vez.
pub(crate) trait RequestFactory {
    /// O DTO que esta factory produz.
    ///
    /// `Send` porque atravessa o `.await` do caso de uso.
    type Message: Send;

    /// Lê a mensagem de um objeto JSON já parseado.
    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError>;

    /// Lê a mensagem de um buffer `FlatBuffers`.
    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError>;
}
