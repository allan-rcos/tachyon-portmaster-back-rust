//! Os testes de `app_error`.

use super::*;

#[test]
fn o_slug_negado_fica_no_erro() {
    // Vai para o log; quem decide não mostrá-lo ao cliente é o api-http.
    let error = AppError::permission_denied("product:create");

    assert!(error.to_string().contains("product:create"));
}
