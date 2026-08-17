//! A pilha de eventos por tarefa.
//!
//! **O canal por onde um service fala com um middleware.** Um caso de uso
//! registra um fato sobre como produziu a resposta, e um middleware pergunta
//! depois o que foi registrado. Entre os dois não há nada: o caso de uso não
//! devolve o evento, o controller não o repassa, e nenhuma assinatura no meio
//! muda quando um evento novo aparece.
//!
//! É para isso que ela serve, e não só para o cache. `ViewCacheHit` é o primeiro
//! [`MetaEvent`] porque foi o primeiro a precisar; qualquer coisa que a borda
//! precise saber sobre como a resposta foi produzida entra pelo mesmo caminho,
//! sem tocar em quem está entre os dois.
//!
//! ## Emitir é privado, ler é público
//!
//! O contrato de escrita não sai do crate — só um caso de uso emite, e quem está
//! fora não consegue nem nomear o que precisaria implementar.
//! [`MetaEventStackSubscriber`] é público, porque quem abre o escopo e quem lê o
//! resultado é a camada de cima.
//!
//! ## O escopo é opcional, sempre
//!
//! Sem ele, emitir não faz nada e perguntar responde `false`. Um caso de uso
//! chamado de um teste, de um comando ou de uma tarefa de fundo funciona igual —
//! o que se perde é só a resposta da pergunta, que ninguém estava fazendo.
//!
//! Ver `docs/adr/0013-pilha-de-eventos-por-tarefa.md` para por que isto existe
//! ao lado do span do `tracing`.

pub mod meta_event;
pub mod meta_event_stack_subscriber;

pub(crate) mod event_provider;
pub(crate) mod meta_event_stack_publisher;

pub(crate) mod intern;

pub use meta_event::MetaEvent;
pub use meta_event_stack_subscriber::MetaEventStackSubscriber;

pub(crate) use event_provider::EventProvider;
pub(crate) use meta_event_stack_publisher::MetaEventStackPublisher;
