//! O que um contexto por tarefa sabe fazer.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;

/// O futuro de fechar um contexto.
///
/// Boxeado porque a lista de contextos é heterogênea: o `Arc<dyn ScopeContext>`
/// que o iterador percorre precisa de um tipo de retorno só, e `async fn` numa
/// trait dá um futuro opaco diferente por implementação. `pub(super)` para não
/// competir com o export do arquivo — ele é auxiliar, não contrato.
pub(super) type Closing<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// Um estado que nasce e morre com a tarefa, e que sabe confirmar-se.
///
/// É o que a camada põe no mapa do escopo: o mesmo objeto guarda o estado (a
/// transação, o buffer de escritas) e sabe fechá-lo. Não há `begin` aqui de
/// propósito — abrir é a única operação que precisa do recurso caro, e o objeto
/// que o `install` constrói nasce de um `fn` resolvido pelo linker, sem provider
/// de onde tirá-lo. Quem abre é o handle injetado nos repositórios.
///
/// `Any` é o que permite ao dono recuperar o próprio contexto com o tipo de
/// volta, depois de ele ter entrado na lista como `dyn`.
pub(crate) trait ScopeContext: Any + Send + Sync {
    /// Confirma o que a tarefa acumulou.
    ///
    /// Sem nada acumulado é no-op: um caso de uso que só leu não abriu
    /// transação, e confirmar o nada não é erro.
    fn commit(&self) -> Closing<'_>;

    /// Descarta o que a tarefa acumulou.
    ///
    /// Idempotente, e no-op depois do commit — o escopo o chama na saída sem
    /// saber se o corpo confirmou.
    fn rollback(&self) -> Closing<'_>;
}
