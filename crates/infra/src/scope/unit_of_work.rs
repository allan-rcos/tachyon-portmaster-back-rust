//! O contrato da unidade de trabalho.

/// Fecha o que a tarefa acumulou — e nada mais.
///
/// Não executa consulta: o que ela oferece é fim e desistência. Manter a
/// execução fora daqui é o que impede o `app` de rodar SQL.
///
/// Não há `begin`. Abrir é a única das três operações que precisa do recurso
/// caro, e quem tem o pool em mãos é o handle injetado nos repositórios, não o
/// objeto que o escopo monta a partir da slice. Então abrir virou detalhe de
/// quem escreve: o primeiro repositório a pedir a transação a abre, e um caso de
/// uso que só leu da memória nunca abriu nenhuma.
#[trait_variant::make(Send)]
pub trait UnitOfWork {
    /// Confirma o que foi feito.
    async fn commit(&self) -> anyhow::Result<()>;

    /// Descarta o que foi feito.
    ///
    /// Idempotente: chamar sem nada aberto não é erro, porque o caminho de falha
    /// frequentemente não sabe se chegou a abrir alguma coisa.
    async fn rollback(&self) -> anyhow::Result<()>;
}
