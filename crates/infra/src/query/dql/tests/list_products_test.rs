//! Os testes de `list_products`.

use super::*;
use pretty_assertions::assert_eq;

/// O SQL que um DQL monta, para as asserções.
fn sql_of(dql: &impl SqlDql) -> String {
    dql.build().sql().as_str().to_owned()
}

/// O total tem que descrever o mesmo conjunto que a página percorre — senão
/// uma busca por uma palavra reportaria o catálogo inteiro.
#[test]
fn a_busca_entra_na_pagina_e_na_contagem() {
    let dql = ListProducts {
        limit: 20,
        search: Some("cimento".into()),
        cursor: None,
    };

    assert_eq!(
        sql_of(&dql),
        "SELECT p.id, p.name, p.density, p.risk_class, \
         (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND search_name LIKE ?) AS _total \
         FROM products p WHERE p.id > ? AND p.deleted_at IS NULL AND p.search_name LIKE ? \
         ORDER BY p.id ASC LIMIT ?"
    );
}

#[test]
fn sem_busca_nao_ha_filtro_de_texto() {
    let dql = ListProducts {
        limit: 20,
        search: None,
        cursor: None,
    };

    assert!(!sql_of(&dql).contains("LIKE"));
}

/// Trocar o termo e reenviar o cursor antigo continuaria a varredura do
/// conjunto anterior sob o filtro novo.
#[test]
fn um_cursor_de_outra_busca_recomeca_do_zero() {
    let anterior = ListProducts {
        limit: 20,
        search: Some("cimento".into()),
        cursor: None,
    };
    let token =
        Cursor::next(20, 20, 900, &anterior.cursor_filters()).expect("página cheia emite cursor");

    let outra = ListProducts {
        limit: 20,
        search: Some("areia".into()),
        cursor: Some(token.clone()),
    };
    let mesma = ListProducts {
        limit: 20,
        search: Some("cimento".into()),
        cursor: Some(token),
    };

    assert_eq!(
        Cursor::last_id_or_start(outra.cursor.as_deref(), &outra.cursor_filters()),
        0,
        "o cursor de outra busca deveria ter sido ignorado"
    );
    assert_eq!(
        Cursor::last_id_or_start(mesma.cursor.as_deref(), &mesma.cursor_filters()),
        900,
        "o cursor da mesma busca move o piso da varredura"
    );
}

#[test]
fn o_limite_ausente_ou_zero_cai_no_padrao() {
    for limit in [None, Some(0)] {
        assert_eq!(
            Paging::effective_limit(limit),
            crate::query::DEFAULT_LIMIT,
            "limite {limit:?} deveria cair no padrão"
        );
    }
}
