//! Os testes de `env_source`.

use super::*;
use pretty_assertions::assert_eq;

/// Uma fonte sobre um punhado de variáveis.
fn source(pairs: &[(&str, &str)]) -> EnvSource {
    EnvSource::of_pairs(
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())),
    )
}

#[test]
fn variavel_vazia_conta_como_ausente() {
    assert_eq!(
        source(&[("APP_HOST", "   ")]).string("APP_HOST", "padrão"),
        "padrão"
    );
}

#[test]
fn numero_ilegivel_e_queixa_e_nao_padrao() {
    let mut env = source(&[("APP_PORT", "oito mil")]);

    assert_eq!(env.number("APP_PORT", 8000_u16), 8000);
    assert!(env.into_result().is_err());
}

#[test]
fn o_booleano_so_e_verdadeiro_nas_grafias_afirmativas() {
    for afirmativo in ["1", "true", "TRUE", "yes", "on"] {
        assert!(
            source(&[("F", afirmativo)]).flag("F", false),
            "{afirmativo}"
        );
    }

    for negativo in ["0", "false", "no", "qualquer coisa"] {
        assert!(!source(&[("F", negativo)]).flag("F", true), "{negativo}");
    }
}

#[test]
fn a_lista_ignora_espaco_e_entrada_vazia() {
    assert_eq!(
        source(&[("O", "https://a.test, ,https://b.test ")]).list("O", &[]),
        ["https://a.test".to_owned(), "https://b.test".to_owned()]
    );
}

/// É o ponto do desenho: um boot recusado conta tudo de uma vez.
#[test]
fn as_queixas_se_acumulam_em_vez_de_parar_na_primeira() {
    let mut env = source(&[]);
    env.required("PRIMEIRA");
    env.required("SEGUNDA");

    let message = env
        .into_result()
        .expect_err("duas ausências recusam o boot")
        .to_string();

    assert!(message.contains("PRIMEIRA"), "{message}");
    assert!(message.contains("SEGUNDA"), "{message}");
}
