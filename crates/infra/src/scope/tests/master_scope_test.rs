//! Os testes de `master_scope`.

use std::sync::{Arc, Mutex};

use linkme::distributed_slice;

use super::*;
use crate::scope::intern::scope_slots::CURRENT;
use crate::scope::scope_context::{Closing, ScopeContext};
use crate::scope::scope_layer::ScopeLayer;

/// O que a sonda registrou num escopo, na ordem.
type Journal = Arc<Mutex<Vec<&'static str>>>;

/// Um contexto que só anota o que lhe pediram.
///
/// O diário é **do escopo**, e não global: a sonda entra em todo escopo que
/// este binário de teste abrir, e um diário compartilhado misturaria o
/// fecho de um teste com o do outro sob execução paralela.
struct ProbeContext {
    /// O diário deste escopo.
    journal: Journal,
}

impl ProbeContext {
    /// Registra a sonda na tarefa.
    fn install(slots: &mut ScopeSlots) {
        slots.put(Self {
            journal: Journal::default(),
        });
    }

    /// O diário do escopo corrente.
    fn journal() -> Journal {
        ScopeSlots::current::<Self>()
            .expect("a sonda está instalada")
            .journal
            .clone()
    }
}

impl ScopeContext for ProbeContext {
    fn commit(&self) -> Closing<'_> {
        Box::pin(async {
            self.journal.lock().expect("diário").push("commit");
            Ok(())
        })
    }

    fn rollback(&self) -> Closing<'_> {
        Box::pin(async {
            self.journal.lock().expect("diário").push("rollback");
            Ok(())
        })
    }
}

#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio é local ao registro e não sobe para o lib.rs"
)]
#[distributed_slice(SCOPE_LAYERS)]
static PROBE: ScopeLayer = ScopeLayer {
    install: ProbeContext::install,
};

/// A slice chega populada no binário de teste.
///
/// É o modo de falha clássico do `linkme`, e ele falha **silencioso**: um
/// escopo sem camada nenhuma não acusa nada, só deixa de confirmar.
#[test]
fn a_slice_chega_populada() {
    assert!(
        !SCOPE_LAYERS.is_empty(),
        "o linker manteve as seções das camadas"
    );
}

/// Fora de um escopo não há onde guardar contexto.
#[tokio::test]
async fn fora_do_escopo_nao_ha_contexto() {
    assert!(!MasterScope::is_active());
    assert!(ScopeSlots::current::<ProbeContext>().is_err());
}

/// Dentro dele, a camada declarada está lá — sem ninguém a ter importado.
#[tokio::test]
async fn dentro_do_escopo_a_camada_declarada_esta_instalada() {
    MasterScope::run(|_| async {
        assert!(MasterScope::is_active());
        assert!(ScopeSlots::current::<ProbeContext>().is_ok());
    })
    .await;
}

/// Confirmar é do corpo; desfazer, da saída — e ela acontece de todo jeito.
///
/// O `?` volta a ser `?`: o braço de erro não chama rollback, e mesmo assim
/// o contexto é desfeito.
#[tokio::test]
async fn o_corpo_confirma_e_a_saida_desfaz() {
    let confirmed = MasterScope::run(|uow| async move {
        let journal = ProbeContext::journal();
        uow.commit().await.expect("commit da sonda não falha");
        journal
    })
    .await;

    assert_eq!(
        confirmed.lock().expect("diário").as_slice(),
        ["commit", "rollback"],
        "o commit do corpo, e o fecho do escopo — que nada mais tem a desfazer"
    );

    let (result, abandoned): (Result<(), anyhow::Error>, Journal) = MasterScope::run(|_| async {
        let journal = ProbeContext::journal();
        (Err(anyhow::anyhow!("o corpo desistiu")), journal)
    })
    .await;

    assert!(result.is_err());
    assert_eq!(
        abandoned.lock().expect("diário").as_slice(),
        ["rollback"],
        "ninguém chamou rollback: sair por erro já é o rollback"
    );
}

/// A garantia que sustenta o modelo: duas requisições simultâneas têm
/// contextos independentes, sem lock global entre elas.
///
/// Os dois `Arc` são segurados vivos ao mesmo tempo de propósito. Se cada
/// tarefa apenas devolvesse um ponteiro, a primeira alocação já estaria
/// liberada quando a segunda acontecesse, e o alocador poderia devolver o
/// mesmo endereço — o teste acusaria uma mistura que não houve.
#[tokio::test]
async fn escopos_de_tarefas_diferentes_nao_se_misturam() {
    let first = tokio::spawn(MasterScope::run(|_| async { CURRENT.with(Arc::clone) }));
    let second = tokio::spawn(MasterScope::run(|_| async { CURRENT.with(Arc::clone) }));

    let (first, second) = tokio::join!(first, second);
    let first = first.expect("tarefa não deve entrar em pânico");
    let second = second.expect("tarefa não deve entrar em pânico");

    assert!(
        !Arc::ptr_eq(&first, &second),
        "cada escopo tem o próprio armazenamento"
    );
}
