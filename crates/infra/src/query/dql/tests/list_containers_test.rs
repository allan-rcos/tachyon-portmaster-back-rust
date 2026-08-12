//! Os testes de `list_containers`.

use super::*;

fn dql(status: Option<ContainerStatus>, status_in: Vec<ContainerStatus>) -> ListContainers {
    ListContainers {
        limit: 20,
        search: None,
        status,
        status_in,
        cursor: None,
    }
}

#[test]
fn o_conjunto_de_status_vira_um_in_com_um_placeholder_por_valor() {
    let sql = dql(
        None,
        vec![ContainerStatus::Loading, ContainerStatus::Sealed],
    )
    .build()
    .sql()
    .as_str()
    .to_owned();

    assert!(
        sql.contains("c.status IN (?, ?)"),
        "esperava um placeholder por status: {sql}"
    );
}

/// Se divergirem, o cliente recebe uma página de três itens dizendo que há
/// quatrocentos.
#[test]
fn a_contagem_repete_exatamente_os_filtros_da_pagina() {
    let dql = ListContainers {
        limit: 20,
        search: Some("br-99".into()),
        status: Some(ContainerStatus::InTransit),
        status_in: Vec::new(),
        cursor: None,
    };

    let sql = dql.build().sql().as_str().to_owned();

    for predicate in ["search_code LIKE ?", "status = ?"] {
        assert!(
            sql.contains(&format!(" AND {predicate}")),
            "a contagem perdeu `{predicate}`: {sql}"
        );
        assert!(
            sql.contains(&format!(" AND c.{predicate}")),
            "a página perdeu `{predicate}`: {sql}"
        );
    }
}
