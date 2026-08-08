//! A primitiva de marcação.

use crate::commands::marker::SetMarkerCommand;
use crate::error::AppError;
use crate::queries::marker::GetMarkerQuery;

/// A primitiva de marcação.
#[trait_variant::make(Send)]
pub trait MarkUseCase {
    /// Marca um valor, aplicando as regras de transição do grupo.
    async fn set(&self, command: SetMarkerCommand) -> Result<(), AppError>;

    /// Se a marca vale agora.
    ///
    /// Marca inexistente, expirada ou desligada respondem igual: `false`. Quem
    /// pergunta quer saber se pode seguir, não por que não pode — e distinguir
    /// os casos diria a quem tentasse adivinhar quais valores já existiram.
    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, AppError>;
}
