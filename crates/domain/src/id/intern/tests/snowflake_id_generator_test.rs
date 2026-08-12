//! Os testes de `snowflake_id_generator`.

use super::*;
use std::collections::HashSet;

/// O caso que o gerador atômico existe para cobrir.
///
/// Várias threads emitindo dentro do mesmo milissegundo.
#[test]
fn ids_sao_unicos_sob_concorrencia() {
    let generator = SnowflakeIdGenerator::new(1, 1);
    let mut handles = Vec::new();

    for _ in 0..8 {
        let generator = generator.clone();
        handles.push(std::thread::spawn(move || {
            (0..500).map(|_| generator.next()).collect::<Vec<_>>()
        }));
    }

    let mut all = HashSet::new();
    for handle in handles {
        for id in handle.join().expect("thread de emissão entrou em pânico") {
            assert!(all.insert(id.clone()), "id repetido: {id}");
        }
    }

    assert_eq!(all.len(), 4_000);
}

#[test]
fn ids_crescem_com_o_tempo() {
    let generator = SnowflakeIdGenerator::new(1, 1);
    let first = base62::decode(generator.next()).expect("id emitido deve decodificar");
    let second = base62::decode(generator.next()).expect("id emitido deve decodificar");

    assert!(second > first, "{second} deveria vir depois de {first}");
}

/// Dois deploys diferentes não podem cair no mesmo `instance`, senão os
/// dois emitem a mesma sequência dentro do mesmo milissegundo.
#[test]
fn cada_par_cluster_servidor_tem_o_seu_instance() {
    let mut vistos = HashSet::new();

    for cluster in 0..32 {
        for server in 0..32 {
            assert!(
                vistos.insert(instance_of(cluster, server)),
                "cluster {cluster}/servidor {server} colidiu com um par anterior"
            );
        }
    }

    assert_eq!(
        vistos.len(),
        1_024,
        "os dez bits de instance ficaram ociosos"
    );
}

/// Um segredo fora da faixa é preso na borda em vez de derrubar o boot.
#[test]
fn valor_fora_da_faixa_e_preso_na_borda() {
    assert_eq!(instance_of(-1, -1), 0);
    assert_eq!(instance_of(999, 999), 1_023);
}
