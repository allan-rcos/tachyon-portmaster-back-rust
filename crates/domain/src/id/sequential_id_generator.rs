//! O contrato do id ordenável no tempo.

/// Gera um id ordenável em string.
///
/// O `request_id` do logger é o caso: precisa ordenar por tempo para que os logs
/// de uma requisição se sequenciem, mas nunca vira chave primária — e por isso
/// não paga o preço de um gerador de banco.
///
/// Pede `Clone + Send + Sync + 'static` porque quem o consome é um
/// `tower::Layer`, e o axum exige que um layer seja clonável e compartilhável
/// entre tarefas. Exigir aqui, e não no ponto de uso, é o que evita a
/// apresentação descobrir a restrição como um erro de trait a três camadas de
/// distância — o gerador não tem estado, então não paga nada por isso.
pub trait SequentialIdGenerator: Clone + Send + Sync + 'static {
    /// Um id novo, ordenável lexicograficamente.
    fn next(&self) -> String;
}
