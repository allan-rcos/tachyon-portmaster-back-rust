//! O contrato entre um VO de requisição e os DTOs que chegam do fio.

use serde::de::DeserializeOwned;

use crate::ports::error::api_error::ApiError;

/// Um VO de requisição que sabe nascer de cada um dos DTOs de entrada.
///
/// É a simetria do [`ResponseX`](super::response_x::ResponseX): o controller
/// recebe o VO e não sabe por qual formato ele chegou.
///
/// ## Os campos são `Option`, e é de propósito
///
/// O `.fbs` marca vários campos `required`, mas o VO os recebe como `Option`.
/// Um campo ausente chega como `None`, o caso de uso o repassa ao `TableModule`,
/// e o cliente recebe **422 nomeando o campo** — todos os que faltaram, de uma
/// vez. Se o DTO de JSON exigisse o campo, o serde falharia antes de o
/// controller existir e a resposta seria um 400 sem dizer o quê.
///
/// É também o que torna desnecessário mexer no `.fbs`: o `required` de lá
/// restringe a tabela do planus, não o DTO de JSON, que é nosso.
pub(crate) trait RequestX: Sized + Send {
    /// O DTO de JSON, com todo campo opcional.
    type Json: DeserializeOwned;

    /// O VO a partir do que o serde leu.
    fn of_json(dto: Self::Json) -> Self;

    /// O VO a partir do buffer binário.
    ///
    /// Recebe os bytes crus, e não uma tabela já lida, porque ler a raiz é o que
    /// pode falhar: um buffer truncado é ilegível, e só quem sabe qual tabela
    /// esperar consegue dizer isso.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError>;
}
