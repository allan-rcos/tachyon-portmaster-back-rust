//! Os testes de `list_container_summaries`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_janela_de_telemetria_e_correlacionada_por_id() {
    // O MariaDB não tem LATERAL: uma tabela derivada não enxergaria `c.id`.
    let dql = ListContainerSummaries {
        limit: 20,
        id: None,
        raw_id: None,
        cursor: None,
    };
    let sql = dql.build().sql().as_str().to_owned();

    assert!(
        sql.contains("LIMIT 1 OFFSET 9"),
        "a janela deveria limitar pelo décimo mais novo: {sql}"
    );
    assert!(
        sql.contains("COALESCE("),
        "faltou o caso de menos logs que o teto"
    );
}

/// `JSON_ARRAYAGG` devolve `NULL` num contêiner sem carga, que é estado
/// normal.
#[test]
fn manifesto_vazio_nao_e_erro() {
    assert_eq!(read_manifest(None).unwrap(), Vec::new());
}

#[test]
fn o_manifesto_sai_do_json_agregado() {
    let json = r#"[{"product_id":1,"product_name":"Cimento","quantity":2.5,"weight":50.0}]"#;

    assert_eq!(
        read_manifest(Some(json)).unwrap(),
        vec![CargoItemView {
            product_id: Codec::encode_id(1),
            product_name: "Cimento".into(),
            quantity: 2.5,
            weight: 50.0,
        }]
    );
}

/// O campo do fio é um enum: não há valor que signifique "aconteceu algo,
/// mas nenhum destes".
#[test]
fn evento_desconhecido_e_descartado_e_nao_aproximado() {
    let json = r#"[{"id":1,"event":0,"description":null,"timestamp":1000},
                   {"id":2,"event":98,"description":null,"timestamp":2000}]"#;

    let logs = read_logs(Some(json)).unwrap();

    assert_eq!(logs.len(), 1, "o evento fora da faixa deveria ter saído");
    assert_eq!(logs[0].event, TelemetryEvent::Load.as_i32());
    assert_eq!(logs[0].timestamp, 1000);
}
