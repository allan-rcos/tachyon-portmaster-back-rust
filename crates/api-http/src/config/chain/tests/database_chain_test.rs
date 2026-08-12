//! Os testes de `database_chain`.

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

/// O mesmo modo aparece escrito de formas diferentes em compose, shell e
/// documentação; recusar por causa de um hífen seria hostilidade.
#[test]
fn o_modo_de_tls_aceita_as_grafias_usuais() {
    for grafia in ["verify_ca", "verify-ca", "VERIFY_CA"] {
        let mut env = source(&[(Env::DB_SSL_MODE, grafia)]);

        assert_eq!(DatabaseChain::ssl_mode(&mut env), DatabaseSslMode::VerifyCa);
        assert!(env.into_result().is_ok(), "{grafia}");
    }
}

#[test]
fn um_modo_de_tls_desconhecido_derruba_o_boot() {
    let mut env = source(&[(Env::DB_SSL_MODE, "mais_ou_menos")]);
    let _ = DatabaseChain::ssl_mode(&mut env);

    assert!(env.into_result().is_err());
}
