//! O contrato de quem emite identidade de entidade.

/// Gera o id de uma entidade persistida, já compactado em base62.
///
/// O `&self` não é um detalhe: o gerador é compartilhado por todas as threads do
/// processo, então a impl guarda o seu estado atrás de um lock. É diferente do
/// modelo de processos forkados, em que cada worker tinha o seu próprio contador
/// e a unicidade dependia de nunca repetir o par cluster/server.
pub(crate) trait IntIdGenerator {
    /// Um id novo, único e crescente no tempo.
    fn next(&self) -> String;
}
