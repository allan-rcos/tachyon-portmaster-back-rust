//! O que uma camada declara para participar do escopo.

use crate::scope::intern::scope_slots::ScopeSlots;

/// A declaração de uma camada na slice do escopo.
///
/// Um campo só, e é o que mantém a declaração de um contexto novo em uma linha.
/// A alternativa — um ponteiro de função por operação (`init`, `commit`,
/// `rollback`) — obrigaria cada camada a expor as operações da unidade de
/// trabalho como funções associadas sem `self`, que é justamente o que elas não
/// são: confirmar é sobre o estado daquela tarefa, e o estado tem dono.
pub(crate) struct ScopeLayer {
    /// Constrói o contexto desta camada e o registra na tarefa.
    ///
    /// Síncrono e infalível de propósito: o estado inicial de todo contexto é
    /// vazio, o que não precisa de I/O. Recurso caro só aparece na primeira
    /// escrita, e aí quem o tem em mãos é o handle injetado no repositório.
    pub(crate) install: fn(&mut ScopeSlots),
}
