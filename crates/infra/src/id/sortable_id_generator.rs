//! O contrato do id ordenável no tempo.

/// Gera um id ordenável em string.
///
/// O `request_id` do logger é o caso: precisa ordenar por tempo para que os logs
/// de uma requisição se sequenciem, mas nunca vira chave primária.
pub trait SortableIdGenerator: Clone + Send + Sync + 'static {
    /// Um id novo, ordenável lexicograficamente.
    fn next(&self) -> String;
}
