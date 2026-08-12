//! Os testes de `search_key`.

use super::*;
use pretty_assertions::assert_eq;

fn key(value: &str) -> String {
    SearchKey::of(value)
}

#[test]
fn baixa_a_caixa() {
    assert_eq!(key("Soja Tipo 2"), "soja tipo 2");
}

#[test]
fn colapsa_espaco() {
    assert_eq!(key("  soja   tipo  2  "), "soja tipo 2");
    assert_eq!(key("soja\ttipo\n2"), "soja tipo 2");
}

/// É o ponto da coluna auxiliar: quem busca "acucar" precisa achar
/// "Açúcar".
#[test]
fn tira_acento() {
    assert_eq!(key("Açúcar"), "acucar");
    assert_eq!(key("Óleo de Soja"), "oleo de soja");
    assert_eq!(key("PIÑA"), "pina");
}

#[test]
fn texto_vazio_vira_chave_vazia() {
    assert_eq!(key(""), "");
    assert_eq!(key("   "), "");
}

/// Transliterar tornaria o registro inencontrável para quem o digitou como
/// ele é — que é o motivo de a normalização ser Unicode e não ASCII.
#[test]
fn preserva_alfabeto_que_nao_sabe_decompor() {
    assert_eq!(key("Ячмень"), "ячмень");
    assert_eq!(key("大豆 A"), "大豆 a");
}
