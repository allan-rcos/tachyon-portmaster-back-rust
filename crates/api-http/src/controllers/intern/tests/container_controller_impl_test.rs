//! Os testes de `container_controller_impl`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn o_slug_vira_status() {
    assert_eq!(status_of("in-transit"), Some(ContainerStatus::InTransit));
    assert_eq!(status_of("  SEALED "), Some(ContainerStatus::Sealed));
}

/// Um filtro que não dá para interpretar não deveria esvaziar a listagem.
#[test]
fn slug_desconhecido_nao_filtra() {
    assert_eq!(status_of("carregando"), None);
}
