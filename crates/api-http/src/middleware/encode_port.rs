//! O contrato de quem escreve a resposta no formato negociado.

use axum::http::StatusCode;
use axum::response::Response;

use crate::wire::x::response_x::ResponseX;

/// Escreve um VO de resposta no formato que esta requisição negociou.
///
/// Genérica sobre o VO, e por isso **não é object-safe** — que é o ponto. Cada
/// par (formato, VO) é monomorfizado: o que varia em tempo de execução é só a
/// variante de formato guardada no escopo, e escolher entre elas é um `match`,
/// não uma vTable.
///
/// Repare no que **não** está aqui: nada que revele o formato. Quem tem esta
/// porta na mão consegue responder, e é só isso — não descobre em que formato
/// nem escolhe outro. Escolher é do middleware, uma vez por requisição.
///
/// ## Por que ela substituiu um extractor
///
/// O `Encoder` era um extractor que toda rota declarava e repassava para o
/// `ApiResponse`. Trinta assinaturas carregando um argumento que ninguém lia, só
/// para levá-lo do axum até o construtor da resposta. Agora o formato está no
/// escopo da requisição, e quem responde pede a porta.
pub(crate) trait EncodePort: Clone + Send + Sync + 'static {
    /// A resposta completa: corpo codificado e tipo de mídia.
    ///
    /// É o único jeito de tirar bytes daqui, e é de propósito: se codificar e
    /// carimbar o cabeçalho fossem duas chamadas, existiria um caminho em que a
    /// primeira acontece sem a segunda — um corpo saindo com o tipo de mídia
    /// errado, que é exatamente o defeito que este desenho fecha.
    ///
    /// Cookie não passa por aqui. Quem os carimba é o layer de cookie, depois do
    /// handler e para toda resposta — inclusive a de erro.
    ///
    /// Falha na serialização vira `502` e sai como corpo de problema — pelo
    /// mesmo formato, para que nem o erro do erro escape da negociação.
    fn respond<X: ResponseX>(&self, status: StatusCode, body: &X) -> Response;
}
