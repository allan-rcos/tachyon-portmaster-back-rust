//! Metadados de sistema: permissões e grupos de marcador.
//!
//! São tipos de domínio **imutáveis e extensíveis**: crescem conforme o `app`
//! ganha casos de uso, mas depois do boot nunca mudam. São estado do sistema
//! para o sistema — não dado de negócio.
//!
//! Três consequências de serem assim:
//!
//! * **Não são enum.** Um enum é fechado no código, e estes crescem a cada caso
//!   de uso novo — seria editar o `domain` a cada feature.
//! * **Não são linha de banco de negócio.** O valor é fixo após o boot e é lido
//!   a cada requisição; o backing natural é cache.
//! * **Não têm timestamps.** São a única exceção à regra de que todo objeto de
//!   domínio carrega ao menos `created_at`, porque não têm ciclo de vida de
//!   linha: são registrados no boot e pronto.
//!
//! O `domain` declara o **tipo** e valida o formato do slug. Quem registra é o
//! `app`, no boot; quem guarda é a `infra`, em cache.

pub mod marker_group;
pub mod permission;

use crate::error::{MetadataError, Validation};

/// Comprimento máximo de um slug.
///
/// Casa com o limite que o modelo anterior validava, e mantém a chave curta —
/// ela é lida a cada verificação de permissão.
pub(crate) const MAX_SLUG_LENGTH: usize = 64;

/// Valida um token em lower-kebab: `refresh-token`, `metrics`.
///
/// Começa por letra minúscula, segue com minúsculas e dígitos, e admite hífen
/// entre segmentos — nunca no começo, no fim, nem dobrado.
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
/// `shape` descreve o formato esperado na mensagem, para que permissão e grupo
/// digam ao cliente coisas diferentes sem duplicar a lógica.
pub(crate) fn validate_slug(slug: &str, valid: bool, shape: &str) -> Result<(), MetadataError> {
    let mut errors = Validation::new();

    if slug.is_empty() {
        errors.add("slug", "Slug is required.");
    } else if !valid {
        errors.add("slug", shape);
    } else if slug.chars().count() > MAX_SLUG_LENGTH {
        errors.add(
            "slug",
            format!("Slug must not exceed {MAX_SLUG_LENGTH} characters."),
        );
    }

    errors.into_result(()).map_err(MetadataError::Validation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_tokens_kebab_validos() {
        for good in ["metrics", "refresh-token", "rate-limit", "a1", "x-9-y"] {
            assert!(is_kebab_token(good), "deveria aceitar {good}");
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
            assert!(!is_kebab_token(bad), "deveria recusar {bad:?}");
        }
    }
}
