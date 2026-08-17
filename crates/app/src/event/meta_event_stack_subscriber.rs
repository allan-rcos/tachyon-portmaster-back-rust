//! O lado de leitura da pilha de eventos.

use std::future::Future;

use crate::event::meta_event::MetaEvent;

/// Abre o escopo da pilha e consulta o que foi registrado nele.
///
/// Público de propósito, e é a metade pública do par: quem **emite** é um caso
/// de uso, quem **lê** é a apresentação. Um middleware abre o escopo, o caso de
/// uso emite lá dentro, e outro middleware pergunta o que saiu — sem que nenhum
/// dos três conheça os outros.
///
/// ## Sem escopo, nada acontece
///
/// As três operações são idempotentes quanto à existência do escopo: fora dele,
/// [`flush`](Self::flush) não faz nada e [`captured`](Self::captured) responde
/// `false`. É o que permite chamar um caso de uso de um teste, de um comando de
/// linha ou de uma tarefa de fundo sem montar middleware nenhum.
pub trait MetaEventStackSubscriber {
    /// Roda `future` com uma pilha vazia instalada na tarefa.
    ///
    /// Um `task_local` não pode ser escrito de dentro, só envolvido — então esta
    /// é a única forma de a pilha existir. Aninhar dois escopos instala uma
    /// pilha nova: o de dentro não enxerga o que o de fora registrou, e o de
    /// fora não recebe o que o de dentro emitiu.
    ///
    /// Não é `async fn` porque o futuro que embrulha já vem pronto de baixo, e
    /// devolvê-lo direto é o que dispensa a variante `Send` gerada — o `Send`
    /// sai daqui declarado.
    fn scope<F>(&self, future: F) -> impl Future<Output = F::Output> + Send
    where
        F: Future + Send;

    /// Esquece um evento, ou a pilha inteira.
    ///
    /// Com `Some`, só aquele evento deixa de estar registrado; com `None`, a
    /// pilha volta a ser o que era quando o escopo abriu.
    fn flush(&self, event: Option<MetaEvent>);

    /// Se o evento foi registrado nesta tarefa.
    fn captured(&self, event: MetaEvent) -> bool;
}
