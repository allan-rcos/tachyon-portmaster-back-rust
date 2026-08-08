//! A validação de slug que permissão e grupo de marcador compartilham.
//!
//! Um slug é o nome estável de um metadado de sistema — `metrics`,
//! `refresh-token`, `container.create`. Permissão e grupo têm formatos
//! diferentes (uma admite ponto, o outro não), mas as regras de vazio, de
//! comprimento e de como um segmento kebab é formado são as mesmas nos dois.
//! Ficam aqui uma vez só.
//!
//! É um namespace, não um valor: o molde é o `SearchKey`/`Base62` do PHP —
//! consts e funções associadas, sem construtor.

use crate::error::{MetadataError, Validation};

/// Regras de forma de um slug de metadado.
pub(crate) struct Slug;

impl Slug {
    /// Comprimento máximo de um slug.
    ///
    /// Casa com o limite que o modelo anterior validava, e mantém a chave curta
    /// — ela é lida a cada verificação de permissão.
    pub(crate) const MAX_LENGTH: usize = 64;

    /// Valida um token em lower-kebab: `refresh-token`, `metrics`.
    ///
    /// Começa por letra minúscula, segue com minúsculas e dígitos, e admite
    /// hífen entre segmentos — nunca no começo, no fim, nem dobrado.
    pub(crate) fn is_kebab_token(value: &str) -> bool {
        if value.is_empty() {
            return false;
        }

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

    /// Valida um slug composto e devolve os erros acumulados.
    ///
    /// `shape` descreve o formato esperado na mensagem, para que permissão e
    /// grupo digam ao cliente coisas diferentes sem duplicar a lógica.
    pub(crate) fn validate(slug: &str, valid: bool, shape: &str) -> Result<(), MetadataError> {
        let mut errors = Validation::new();

        if slug.is_empty() {
            errors.add("slug", "Slug is required.");
        } else if !valid {
            errors.add("slug", shape);
        } else if slug.chars().count() > Self::MAX_LENGTH {
            errors.add(
                "slug",
                format!("Slug must not exceed {} characters.", Self::MAX_LENGTH),
            );
        }

        errors.into_result(()).map_err(MetadataError::Validation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_tokens_kebab_validos() {
        for good in ["metrics", "refresh-token", "rate-limit", "a1", "x-9-y"] {
            assert!(Slug::is_kebab_token(good), "deveria aceitar {good}");
        }
    }

    #[test]
    fn recusa_o_que_nao_e_kebab() {
        for bad in [
            "",
            "-leading",
            "trailing-",
            "double--hyphen",
            "Upper",
            "9leading",
            "with_underscore",
            "with space",
        ] {
            assert!(!Slug::is_kebab_token(bad), "deveria recusar {bad:?}");
        }
    }
}
