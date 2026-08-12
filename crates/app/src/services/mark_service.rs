//! A primitiva de marcação.

use crate::commands::marker::{RegisterMarkerGroupCommand, SetMarkerCommand};
use crate::error::MarkerError;
use crate::queries::marker::GetMarkerQuery;

/// A primitiva de marcação.
///
/// Um booleano com prazo, e nada mais: esta camada não sabe o que é sessão nem o
/// que é refresh token. Quem dá nome a um grupo é a apresentação que o usa, e é
/// ela quem o registra no boot.
#[trait_variant::make(Send)]
pub trait MarkService {
    /// Declara um grupo de marcador.
    ///
    /// Chamada no boot, antes de qualquer marca. O repositório recusa marcar num
    /// grupo que não conhece, então quem for usar um grupo precisa declará-lo —
    /// e é quem o usa que sabe o nome dele.
    async fn register_group(&self, command: RegisterMarkerGroupCommand) -> Result<(), MarkerError>;

    /// Marca um valor, aplicando as regras de transição do grupo.
    async fn set(&self, command: SetMarkerCommand) -> Result<(), MarkerError>;

    /// Se a marca vale agora.
    ///
    /// Marca inexistente, expirada ou desligada respondem igual: `false`. Quem
    /// pergunta quer saber se pode seguir, não por que não pode — e distinguir
    /// os casos diria a quem tentasse adivinhar quais valores já existiram.
    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, MarkerError>;
}
