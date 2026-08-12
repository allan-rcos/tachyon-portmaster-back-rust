//! Os testes de `metrics`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn o_painel_sai_de_uma_ida_so_ao_banco() {
    let sql = Metrics.build().sql().as_str().to_owned();

    assert_eq!(
        sql.matches("AS ").count(),
        8,
        "as oito agregações deveriam sair na mesma linha: {sql}"
    );
}

/// Interpolar número em SQL é o hábito que um dia encontra um valor que não
/// é constante.
#[test]
fn os_status_sao_bindados_e_nao_interpolados() {
    let sql = Metrics.build().sql().as_str().to_owned();

    assert_eq!(sql.matches('?').count(), 5, "quatro ocupações e o `<>`");
    assert!(!sql.contains("status = 0"), "índice interpolado: {sql}");
}

#[test]
fn sem_linha_o_painel_sai_zerado() {
    assert_eq!(Metrics.read(Vec::new()).unwrap(), MetricsView::default());
}
