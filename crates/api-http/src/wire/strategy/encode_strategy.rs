//! O contrato de quem escreve uma resposta no fio.

use crate::error::api_error::ApiError;
use crate::wire::x::response_x::ResponseX;

/// Escreve um VO de resposta num formato.
///
/// Genérica sobre o VO, e por isso **não é object-safe** — que é o ponto. Cada
/// par (strategy, VO) é monomorfizado: não há vTable, não há `Arc`, não há
/// alocação para despachar. Quem guarda a strategy da vez é o
/// [`Encoder`](crate::wire::encoder::Encoder), e ele o faz num campo, não num
/// ponteiro.
///
/// Repare no que **não** está aqui: nada que revele o tipo de mídia. Quem usa
/// uma strategy não descobre qual formato ela escreve; isso é assunto dela e de
/// quem a escolheu.
pub(crate) trait EncodeStrategy {
    /// Serializa o VO.
    fn encode<X: ResponseX>(&self, response: &X) -> Result<Vec<u8>, ApiError>;
}
