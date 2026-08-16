//! O id opaco, pelo `NanoID`.

use crate::id::RandomIdGenerator;

/// Comprimento do id aleatório.
///
/// 21 caracteres do alfabeto URL-safe dão cerca de 126 bits de entropia — longe
/// de qualquer força bruta viável, e ainda cabendo num cookie sem incomodar.
const RANDOM_ID_SIZE: usize = 21;

/// Monta o gerador de id opaco.
///
/// O que sai é o contrato, não o tipo: o `NanoID` é detalhe deste arquivo, e
/// quem pede um id opaco só precisa saber que ele é imprevisível.
pub(crate) const fn nano_id_generator() -> impl RandomIdGenerator + use<> {
    NanoIdGenerator
}

/// Gerador de id opaco, sobre `NanoID`.
#[derive(Clone, Copy)]
struct NanoIdGenerator;

impl RandomIdGenerator for NanoIdGenerator {
    fn next(&self) -> String {
        nanoid::nanoid!(RANDOM_ID_SIZE)
    }
}
