//! O contrato de quem emite identidade de entidade.

/// Gera o id de uma entidade persistida, já compactado em base62.
///
/// O id vira chave primária, então ele ordena por tempo: o índice cresce pela
/// ponta em vez de fragmentar. É o requisito que separa este sabor dos outros
/// dois — usá-lo onde o valor nunca chega a uma tabela desperdiça a única coisa
/// que ele tem de caro.
///
/// O `&self` não é um detalhe: o gerador é compartilhado por todas as threads do
/// processo. É diferente do modelo de processos forkados, em que cada worker
/// tinha o seu próprio contador e a unicidade dependia de nunca repetir o par
/// cluster/server.
pub(crate) trait DatabaseIdGenerator {
    /// Um id novo, único e crescente no tempo.
    fn next(&self) -> String;
}
