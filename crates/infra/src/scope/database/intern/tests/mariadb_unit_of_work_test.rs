//! Os testes de `mariadb_unit_of_work`.

use super::*;
use crate::scope::MasterScope;

/// Fora do escopo não há onde guardar a transação, e pedir uma falha em vez
/// de devolver algo inútil.
///
/// É o que mantém o boot funcionando sem cerimônia: o que ele escreve é
/// catálogo em memória, e nada ali pede transação.
#[tokio::test]
async fn fora_do_escopo_nao_ha_contexto_de_transacao() {
    let error = ScopeSlots::current::<MariaDbContext>()
        .err()
        .map(|error| error.to_string())
        .expect("deveria falhar");

    assert!(error.contains("nenhum escopo de tarefa ativo"));
}

/// Abrir o escopo não abre transação: ela só nasce quando alguém escreve.
#[tokio::test]
async fn o_escopo_nasce_sem_transacao() {
    MasterScope::run(|_| async {
        let context = ScopeSlots::current::<MariaDbContext>().expect("o banco está instalado");

        assert!(
            context.slot().lock().await.is_none(),
            "nenhuma consulta pediu a transação ainda"
        );
    })
    .await;
}
