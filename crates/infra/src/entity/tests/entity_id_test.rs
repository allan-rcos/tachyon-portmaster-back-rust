//! Os testes de `entity_id`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_ida_e_a_volta_preservam_o_id() {
    let id = EntityId::try_from(1_234_567_890_i64).expect("id positivo é aceito");

    assert_eq!(id.raw(), 1_234_567_890);
    assert_eq!(
        EntityId::try_from(id.as_str())
            .expect("o base62 que ele mesmo produziu é válido")
            .raw(),
        1_234_567_890
    );
}

/// A codificação acontece uma vez, e a segunda leitura devolve a mesma.
#[test]
fn o_base62_e_codificado_uma_vez_so() {
    let id = EntityId::try_from(42_i64).expect("id positivo é aceito");

    assert!(std::ptr::eq(id.as_str(), id.as_str()));
}

/// Vindo do domínio, o base62 é o que o domínio mandou — não um recodificado.
#[test]
fn o_id_de_dominio_e_guardado_como_veio() {
    let id = EntityId::try_from("aZl8Y0").expect("base62 válido");

    assert_eq!(id.as_str(), "aZl8Y0");
}

/// Id negativo é linha que o schema não deveria admitir, e não vira zero.
#[test]
fn id_negativo_e_recusado() {
    assert!(EntityId::try_from(-1_i64).is_err());
}

#[test]
fn id_fora_do_base62_e_recusado() {
    assert!(EntityId::try_from("não é base62").is_err());
}
