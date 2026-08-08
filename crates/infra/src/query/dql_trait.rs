//! O contrato de uma consulta de leitura.

/// Uma consulta de leitura, independente de backend.
///
/// Amarra só o tipo de saída. É o que permite ao [`crate::query::query_repository::QueryRepository`] devolver
/// `D::View` sem saber o que `View` é.
pub trait Dql {
    /// O read model que esta consulta produz.
    ///
    /// `Send` porque a View atravessa o `.await` da execução e sai por uma
    /// fronteira de tarefa — o handler que a pediu pode estar em outra thread.
    type View: Send;
}
