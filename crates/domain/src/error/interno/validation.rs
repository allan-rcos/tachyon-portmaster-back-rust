//! O acumulador que faz "valide tudo, reclame uma vez" ser o caminho fácil.

use crate::error::FieldError;

/// Acumulador de erros de validação.
///
/// Existe para que o padrão "valide tudo, reclame uma vez" fique difícil de
/// violar por engano: um validator escreve `errors.add(...)` quantas vezes
/// precisar e devolve `errors.into_result(valor)` no fim.
#[derive(Debug, Default, Clone)]
pub(crate) struct Validation {
    /// Os campos recusados até agora, na ordem em que foram conferidos.
    errors: Vec<FieldError>,
}

impl Validation {
    /// Começa uma validação sem erros.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registra um campo inválido e segue examinando os demais.
    pub(crate) fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError::new(field, message));
    }

    /// Registra um campo inválido só se a condição for verdadeira.
    pub(crate) fn add_if(
        &mut self,
        condition: bool,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        if condition {
            self.add(field, message);
        }
    }

    /// Se nada quebrou, devolve o valor; senão, a lista completa de problemas.
    pub(crate) fn into_result<T>(self, value: T) -> Result<T, Vec<FieldError>> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors)
        }
    }
}
