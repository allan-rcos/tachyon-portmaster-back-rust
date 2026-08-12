//! Os testes de `cursor`.

use super::*;
use pretty_assertions::assert_eq;

fn busca(termo: &str) -> CursorFilters {
    CursorFilters::of([("limit", "20".to_owned()), ("search", termo.to_owned())])
}

#[test]
fn ida_e_volta_preserva_o_id() {
    let f = busca("cimento");
    let token = Cursor::encode(9_876_543_210, &f);

    assert_eq!(
        Cursor::decode(Some(&token), &f),
        Some(Cursor {
            last_id: 9_876_543_210
        })
    );
}

/// Base64url sem padding: nada de `+`, `/` ou `=`, que precisariam de
/// escape e voltariam corrompidos.
#[test]
fn o_token_atravessa_uma_querystring_intacto() {
    let token = Cursor::encode(i64::MAX, &busca("álcool etílico 70%"));

    assert!(
        token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "token com caractere que a querystring alteraria: {token}"
    );
}

/// O caso que motiva guardar os filtros: o cliente trocou a busca e
/// reenviou o cursor antigo.
///
/// Continuar dali entregaria uma página do conjunto anterior.
#[test]
fn cursor_de_outro_filtro_e_ignorado() {
    let token = Cursor::encode(100, &busca("cimento"));

    assert_eq!(Cursor::decode(Some(&token), &busca("areia")), None);
}

#[test]
fn toda_recusa_e_a_mesma_ausencia() {
    let f = busca("cimento");

    assert_eq!(Cursor::decode(None, &f), None, "sem token");
    assert_eq!(Cursor::decode(Some(""), &f), None, "token vazio");
    assert_eq!(
        Cursor::decode(Some("!!não é base64!!"), &f),
        None,
        "não é base64url"
    );
    assert_eq!(
        Cursor::decode(Some(&URL_SAFE_NO_PAD.encode("nem json")), &f),
        None,
        "não é o JSON esperado"
    );
}

#[test]
fn sem_cursor_a_varredura_comeca_do_zero() {
    assert_eq!(Cursor::last_id_or_start(None, &busca("cimento")), 0);
}

/// Veio menos que o limite, então acabou.
///
/// Emitir cursor aqui custaria ao cliente uma requisição a mais para
/// descobrir que não há nada.
#[test]
fn pagina_incompleta_nao_emite_proximo_cursor() {
    assert_eq!(Cursor::next(7, 20, 500, &busca("cimento")), None);
}

#[test]
fn pagina_cheia_emite_o_proximo_cursor() {
    let f = busca("cimento");
    let token = Cursor::next(20, 20, 500, &f).expect("página cheia deveria emitir cursor");

    assert_eq!(
        Cursor::decode(Some(&token), &f),
        Some(Cursor { last_id: 500 })
    );
}

/// Dois mapas com os mesmos pares têm que gerar o mesmo token, senão um
/// cursor válido seria recusado por acidente de iteração.
#[test]
fn a_ordem_dos_filtros_nao_muda_o_token() {
    let a = CursorFilters::of([("limit", "20".to_owned()), ("search", "cal".to_owned())]);
    let b = CursorFilters::of([("search", "cal".to_owned()), ("limit", "20".to_owned())]);

    assert_eq!(Cursor::encode(42, &a), Cursor::encode(42, &b));
}
