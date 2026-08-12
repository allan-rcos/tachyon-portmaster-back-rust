//! Os testes de `product_tm_impl`.

use super::*;
use crate::table_modules::intern::helpers::fields_of::fields_of;
use pretty_assertions::assert_eq;

struct FixedIdGenerator;
impl DatabaseIdGenerator for FixedIdGenerator {
    fn next(&self) -> String {
        "P1".to_string()
    }
}

fn table_module() -> ProductTMImpl<FixedIdGenerator> {
    ProductTMImpl::new(FixedIdGenerator)
}

#[test]
fn cria_produto_valido() {
    let product = table_module()
        .create("Soja".into(), 0.75, RiskClass::None)
        .expect("os dados são válidos");

    assert_eq!(product.id(), "P1");
    assert_eq!(product.name(), "Soja");
    assert_eq!(product.density(), 0.75);
    assert_eq!(product.risk_class(), RiskClass::None);
}

#[test]
fn recusa_densidade_que_nao_converte_quantidade_em_peso() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = table_module()
            .create("Soja".into(), bad, RiskClass::None)
            .err()
            .unwrap_or_else(|| panic!("densidade {bad} deveria ser recusada"));

        let ProductError::Validation(fields) = error;
        assert_eq!(fields_of(&fields), vec!["density"], "densidade: {bad}");
    }
}

#[test]
fn acumula_nome_e_densidade_invalidos() {
    let error = table_module()
        .create(String::new(), 0.0, RiskClass::None)
        .err()
        .expect("os dois campos são inválidos");

    let ProductError::Validation(fields) = error;
    assert_eq!(fields_of(&fields), vec!["name", "density"]);
}

#[test]
fn update_nao_altera_o_produto_recebido() {
    let original = table_module()
        .create("Soja".into(), 0.75, RiskClass::None)
        .expect("os dados são válidos");

    let updated = table_module()
        .update(
            original.as_ref(),
            "Soja tipo 2".into(),
            0.8,
            RiskClass::Class9Miscellaneous,
        )
        .expect("os dados são válidos");

    assert_eq!(original.name(), "Soja");
    assert_eq!(original.density(), 0.75);
    assert_eq!(updated.name(), "Soja tipo 2");
    assert_eq!(updated.id(), original.id());
}
