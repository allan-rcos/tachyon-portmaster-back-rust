//! O slug de metadado que permissão, grupo de marcador e marcador compartilham.
//!
//! Um slug é o nome estável de um metadado de sistema — `metrics`,
//! `refresh-token`, `container:create`. Permissão tem um formato composto (dois
//! tokens separados por `:`), os outros dois usam um token só, mas as regras de
//! vazio, de comprimento e de como um segmento kebab é formado são as mesmas nos
//! três. Ficam aqui uma vez só.

use nutype::nutype;

/// Um token de metadado em lower-kebab: `refresh-token`, `metrics`.
///
/// Quem valida um slug composto — a permissão — usa este tipo em cada metade,
/// e confere o todo por conta própria.
#[nutype(validate(not_empty, len_char_max = 64, predicate = Slug::is_kebab_token))]
pub(crate) struct Slug(String);

impl Slug {
    /// Comprimento máximo de um slug.
    ///
    /// Casa com o limite que o modelo anterior validava, e mantém a chave curta
    /// — ela é lida a cada verificação de permissão.
    pub(crate) const MAX_LENGTH: usize = 64;

    /// Se o valor é um token em lower-kebab.
    ///
    /// Começa por letra minúscula, segue com minúsculas e dígitos, e admite
    /// hífen entre segmentos — nunca no começo, no fim, nem dobrado.
    pub(crate) fn is_kebab_token(value: &str) -> bool {
        let mut segments = value.split('-');

        // O primeiro segmento tem a regra extra de começar por letra.
        let Some(first) = segments.next() else {
            return false;
        };
        if !first.starts_with(|c: char| c.is_ascii_lowercase()) {
            return false;
        }
        if !first
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return false;
        }

        segments.all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
    }
}

#[cfg(test)]
#[path = "tests/slug_test.rs"]
mod tests;
