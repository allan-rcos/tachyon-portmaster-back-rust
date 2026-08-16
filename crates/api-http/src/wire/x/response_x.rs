//! O contrato entre um VO de resposta e os DTOs que o representam no fio.

use serde::Serialize;

/// Um VO de resposta que sabe virar cada um dos DTOs de saída.
///
/// ## Os três objetos, e por que são três
///
/// O VO (`…XResponse`) é o **único** que o controller conhece. Ele centraliza os
/// dados da resposta num tipo independente de formato — nem `Serialize`, nem
/// tabela do planus, nem nada que amarre a resposta a como ela sai.
///
/// Os DTOs são dois, um por formato, e **nenhum dos dois é o outro**: o de
/// `FlatBuffers` é a tabela que o planus gera do `.fbs`; o de JSON é uma struct
/// nossa com `#[derive(Serialize)]`. Colar os dois no mesmo tipo — serializar a
/// tabela do planus como JSON — amarraria o corpo JSON à forma do `.fbs`, e uma
/// mudança de schema binário mexeria no contrato textual sem ninguém pedir.
///
/// Esta trait é o que amarra os três. Um terceiro formato é um tipo associado a
/// mais aqui e uma strategy nova — nenhum VO muda, nenhum controller muda.
///
/// ## Por que os métodos não falham
///
/// Converter um VO num DTO é movimento de dados entre dois tipos que já existem;
/// não há o que dar errado. O que pode falhar é **serializar** o DTO, e isso é
/// problema da strategy, que devolve o erro.
pub(crate) trait ResponseX: Send {
    /// O DTO de JSON, gerado pelo `derive` do serde.
    type Json: Serialize;

    /// A tabela do `.fbs`, gerada pelo planus.
    type Fbs: planus::WriteAsOffset<Self::Fbs>;

    /// O VO na forma que o serde escreve.
    fn to_json(&self) -> Self::Json;

    /// O VO na forma que o planus escreve.
    fn to_fbs(&self) -> Self::Fbs;
}
