//! O atalho que os testes de validação usam para ler o lote de erros.
//!
//! Vive num arquivo próprio porque é compartilhado: os testes de usuário, papel,
//! produto e contêiner afirmam todos sobre *quais campos* falharam, e cada um
//! reescrever o `map` seria a mesma linha em quatro lugares.

use crate::error::FieldError;

/// Os nomes dos campos que quebraram, na ordem em que foram acumulados.
pub(crate) fn fields_of(errors: &[FieldError]) -> Vec<&str> {
    errors.iter().map(|e| e.field.as_str()).collect()
}
