//! Os testes de `refresh_token`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn o_refresh_diz_de_quem_e_sem_afirmar_validade() {
    let token = RefreshToken::issue("u1", "V1StGXR8Z5jdHi6BmyT");

    assert_eq!(RefreshToken::owner_of(&token), Some("u1"));
}

#[test]
fn um_refresh_sem_as_duas_partes_nao_tem_dono() {
    // Recusar aqui evita consultar o banco por um id vazio.
    for malformado in ["", "só-id", ".aleatorio", "u1.", "."] {
        assert_eq!(
            RefreshToken::owner_of(malformado),
            None,
            "aceitou {malformado:?}"
        );
    }
}
