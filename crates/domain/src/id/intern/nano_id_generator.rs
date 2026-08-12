//! O id opaco, pelo `NanoID`.

use crate::id::RandomIdGenerator;

/// Comprimento do id aleatório.
///
/// 21 caracteres do alfabeto URL-safe dão cerca de 126 bits de entropia — longe
/// de qualquer força bruta viável, e ainda cabendo num cookie sem incomodar.
const RANDOM_ID_SIZE: usize = 21;

/// Gerador de id opaco, sobre `NanoID`.
#[derive(Clone, Copy)]
pub(crate) struct NanoIdGenerator;

impl NanoIdGenerator {
    /// Monta o gerador.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl RandomIdGenerator for NanoIdGenerator {
    fn next(&self) -> String {
        nanoid::nanoid!(RANDOM_ID_SIZE)
    }
}

#[cfg(test)]
#[path = "tests/nano_id_generator_test.rs"]
mod tests;
