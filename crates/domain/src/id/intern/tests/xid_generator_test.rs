//! Os testes de `xid_generator`.

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
