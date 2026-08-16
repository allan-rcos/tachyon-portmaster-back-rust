//! O id ordenável, pelo xid.

use crate::id::SequentialIdGenerator;

/// Monta o gerador de `request_id`.
///
/// O que sai é o contrato, não o tipo: quem recebe um gerador não tem como
/// descobrir qual algoritmo o emitiu, e trocar o xid por outro não muda nenhuma
/// assinatura fora deste arquivo.
pub(crate) const fn xid_generator() -> impl SequentialIdGenerator + use<> {
    XidGenerator
}

/// Gerador de `request_id`, sobre xid.
#[derive(Clone, Copy)]
struct XidGenerator;

impl SequentialIdGenerator for XidGenerator {
    fn next(&self) -> String {
        xid::new().to_string()
    }
}
