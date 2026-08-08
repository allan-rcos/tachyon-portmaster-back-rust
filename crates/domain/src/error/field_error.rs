//! O campo que quebrou uma regra.

use std::fmt;

/// Um campo que quebrou uma regra, e o porquê.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// Nome do campo, como o cliente o enviou.
    pub field: String,
    /// O que há de errado com ele.
    pub message: String,
}

impl FieldError {
    /// Monta um erro de campo.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}
