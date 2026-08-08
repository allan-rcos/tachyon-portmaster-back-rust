//! O contrato do id opaco e imprevisível.

/// Gera um id opaco e imprevisível.
///
/// O refresh token é o caso: ele precisa ser impossível de adivinhar, que é o
/// oposto do requisito de um id de entidade — este último ordena por tempo
/// justamente para ser previsível ao índice do banco.
/// Os dois pedem `Clone + Send + Sync + 'static` pelo mesmo motivo que
/// [`LoggerFactory`](crate::logging::LoggerFactory): quem os consome é um
/// `tower::Layer`, e o axum exige que um layer seja clonável e compartilhável
/// entre tarefas. Exigir aqui, e não no ponto de uso, é o que evita a
/// apresentação descobrir a restrição como um erro de trait a três camadas de
/// distância — nenhum gerador tem estado, então nenhum paga por isso.
pub trait RandomIdGenerator: Clone + Send + Sync + 'static {
    /// Um id aleatório novo.
    fn next(&self) -> String;
}
