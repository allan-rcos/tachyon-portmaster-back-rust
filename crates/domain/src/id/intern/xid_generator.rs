//! O id ordenável, pelo xid.

use crate::id::SequentialIdGenerator;

/// Gerador de `request_id`, sobre xid.
#[derive(Clone, Copy)]
pub(crate) struct XidGenerator;

impl XidGenerator {
    /// Monta o gerador.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl SequentialIdGenerator for XidGenerator {
    fn next(&self) -> String {
        xid::new().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// É a propriedade que faz os logs se sequenciarem sozinhos quando
    /// ordenados por esse campo.
    #[test]
    fn o_request_id_ordena_pela_emissao() {
        let generator = XidGenerator::new();
        let mut previous = generator.next();

        for _ in 0..100 {
            let current = generator.next();
            assert!(
                current > previous,
                "{current} deveria vir depois de {previous}"
            );
            previous = current;
        }
    }
}
