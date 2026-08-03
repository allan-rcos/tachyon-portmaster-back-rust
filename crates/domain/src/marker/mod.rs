//! O marcador: um booleano com prazo, guardado sob um digest.
//!
//! É uma primitiva do sistema para o sistema, deliberadamente **agnóstica de
//! propósito**. O domínio só sabe que existe um booleano num grupo, identificado
//! por um digest. Que aquilo seja a validade de uma sessão de refresh é
//! conhecimento exclusivo do `api-http` — nem o domínio, nem o `app`, nem a
//! `infra` conhecem JWT.
//!
//! O valor original nunca é guardado: o TableModule recebe o texto em claro e o
//! reduz a um digest. A marca fica leve, e o que está em memória não permite
//! reconstruir o token que a originou.

use crate::error::{MarkerError, Validation};
use crate::metadata::is_kebab_token;
use crate::security::IndexHasher;

/// Um booleano marcado num grupo, sob um digest.
pub trait Marker: Send + Sync {
    /// Slug do grupo a que pertence.
    fn group(&self) -> &str;

    /// Digest do valor marcado — nunca o valor.
    fn key(&self) -> &str;

    /// O booleano em si.
    fn flag(&self) -> bool;
}

/// A implementação do domínio de [`Marker`].
pub(crate) struct MarkerModel {
    group: String,
    key: String,
    flag: bool,
}

impl Marker for MarkerModel {
    fn group(&self) -> &str {
        &self.group
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn flag(&self) -> bool {
        self.flag
    }
}

/// Constrói marcadores, reduzindo o valor em claro a um digest.
pub trait MarkerTM {
    /// Cria um marcador para um valor, num grupo.
    fn create(
        &self,
        group: String,
        plain: &str,
        flag: bool,
    ) -> Result<Box<dyn Marker>, MarkerError>;
}

/// A implementação, genérica sobre o hasher de indexação.
pub(crate) struct MarkerTMImpl<H> {
    hasher: H,
}

impl<H: IndexHasher> MarkerTMImpl<H> {
    /// Monta o TableModule com o seu hasher.
    pub(crate) fn new(hasher: H) -> Self {
        Self { hasher }
    }
}

impl<H: IndexHasher> MarkerTM for MarkerTMImpl<H> {
    fn create(
        &self,
        group: String,
        plain: &str,
        flag: bool,
    ) -> Result<Box<dyn Marker>, MarkerError> {
        let mut errors = Validation::new();

        if group.is_empty() {
            errors.add("group", "Group is required.");
        } else if !is_kebab_token(&group) {
            errors.add(
                "group",
                "Group must be a lower-kebab token (e.g. refresh-token).",
            );
        }

        // Valor vazio hasharia para uma constante, e aí todo chamador que
        // esquecesse de passar algo compartilharia um único marcador — cada um
        // vendo o booleano dos outros virar.
        errors.add_if(plain.is_empty(), "value", "Value is required.");

        errors.into_result(()).map_err(MarkerError::Validation)?;

        Ok(Box::new(MarkerModel {
            group,
            key: self.hasher.hash(plain),
            flag,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::xxhash::XxIndexHasher;
    use pretty_assertions::assert_eq;

    fn table_module() -> MarkerTMImpl<XxIndexHasher> {
        MarkerTMImpl::new(XxIndexHasher::new())
    }

    #[test]
    fn guarda_o_digest_e_nao_o_valor() {
        let marker = table_module()
            .create("refresh-token".into(), "token-secreto", true)
            .expect("grupo e valor são válidos");

        assert_eq!(marker.group(), "refresh-token");
        assert_ne!(marker.key(), "token-secreto");
        assert!(marker.flag());
    }

    #[test]
    fn o_mesmo_valor_reencontra_a_mesma_marca() {
        // É o que faz o refresh funcionar: marcar no login e consultar depois
        // precisam cair na mesma chave.
        let first = table_module()
            .create("refresh-token".into(), "abc", true)
            .expect("válido");
        let second = table_module()
            .create("refresh-token".into(), "abc", false)
            .expect("válido");

        assert_eq!(first.key(), second.key());
    }

    #[test]
    fn recusa_valor_vazio() {
        let error = table_module()
            .create("refresh-token".into(), "", true)
            .err()
            .expect("valor vazio colidiria todo mundo numa marca só");

        let MarkerError::Validation(fields) = error;
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "value");
    }

    #[test]
    fn recusa_grupo_fora_do_formato() {
        for bad in ["", "Refresh-Token", "refresh_token"] {
            assert!(
                table_module().create(bad.into(), "abc", true).is_err(),
                "deveria recusar grupo {bad:?}"
            );
        }
    }
}
