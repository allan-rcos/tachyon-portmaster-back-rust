//! Os testes de `server_controller_impl`.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn o_vm_rss_vira_mib_com_duas_casas() {
    assert_eq!(resident_mib_of("VmRSS:\t   12345 kB\n"), 12.06);
}

/// `/info` é consultada justamente quando algo está estranho; derrubá-la por
/// um formato inesperado seria o pior momento possível.
#[test]
fn um_status_sem_vm_rss_nao_derruba_a_rota() {
    assert_eq!(resident_mib_of("VmPeak:\t 1 kB\n"), 0.0);
}
