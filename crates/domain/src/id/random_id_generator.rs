//! O contrato do id opaco e imprevisível.

/// Gera um id opaco e imprevisível.
///
/// O refresh token é o caso: ele precisa ser impossível de adivinhar, que é o
/// oposto do requisito de um id de entidade — este último ordena por tempo
/// justamente para ser previsível ao índice do banco. Sem timestamp e sem
/// sequência, o valor não conta nada sobre quando ou em que ordem foi emitido.
///
/// Os bounds são os mesmos de [`SequentialIdGenerator`], e pela mesma razão.
///
/// [`SequentialIdGenerator`]: crate::id::SequentialIdGenerator
pub trait RandomIdGenerator: Clone + Send + Sync + 'static {
    /// Um id aleatório novo.
    fn next(&self) -> String;
}
