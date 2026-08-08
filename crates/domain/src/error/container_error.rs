//! O que impede um contêiner de existir ou de mudar de status.

use crate::error::FieldError;

/// Falhas ao construir um contêiner ou ao mover seu status.
///
/// Validação e conflito são coisas diferentes: a primeira diz que os dados
/// enviados estão errados, a segunda que o pátio não está num estado em que
/// aquilo faça sentido.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de contêiner inválidos")]
    Validation(Vec<FieldError>),

    /// Selar exige um contêiner em carregamento.
    #[error("Only a container in the loading state can be sealed.")]
    SealRequiresLoading,

    /// Selar exige carga mínima — senão um contêiner quase vazio sairia como
    /// se fosse um carregamento.
    #[error("A container must be at least 10% full to be sealed.")]
    SealBelowMinimumFill,

    /// Despachar exige um contêiner selado. É também o que torna a operação
    /// idempotente no sentido útil: o segundo despacho é recusado.
    #[error("Only a sealed container can be dispatched.")]
    DispatchRequiresSealed,
}
