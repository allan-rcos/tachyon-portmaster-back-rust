//! O escopo da tarefa, e a unidade de trabalho que o limita.
//!
//! Uma unidade de trabalho não é mais que um limitador de escopo, então ela mora
//! onde o escopo mora — e com ela todo contexto que só vale por tarefa: a
//! transação do banco, o buffer do store em memória, e o que vier depois.
//!
//! ## Os filhos se declaram; o pai não os conhece
//!
//! O [`MasterScope`] não importa nenhum contexto. Cada um se registra em
//! `SCOPE_LAYERS`, que o linker preenche, e o escopo apenas
//! percorre o que encontrar lá.
//!
//! O que isso compra é o que a alternativa custava: um escopo que conhecesse os
//! filhos ganharia uma linha e um nível de aninhamento por contexto novo — ao
//! quinto seriam cinco colunas de indentação, e acrescentar o sexto reescreveria
//! o arquivo inteiro. Aqui um contexto novo é um arquivo novo, e nada mais.
//!
//! ## Quem abre é o caso de uso
//!
//! Não a apresentação. Se o escopo fosse aberto por um middleware do `api-http`,
//! o `app` só funcionaria quando alguém, numa camada que ele nem tem como
//! dependência, tivesse lembrado de declarar aquele middleware — e um segundo
//! transporte teria de redescobrir isso sozinho. Ver
//! `tmp/architecture/request-context.md`.

pub mod master_scope;
pub mod unit_of_work;
pub mod unit_of_work_iterator;

pub(crate) mod database;
pub(crate) mod intern;
pub(crate) mod memory;
pub(crate) mod scope_context;
pub(crate) mod scope_layer;
pub(crate) mod scope_layers;
pub(crate) mod scope_provider;

pub use master_scope::MasterScope;
pub use unit_of_work::UnitOfWork;
pub use unit_of_work_iterator::UnitOfWorkIterator;

pub(crate) use scope_provider::ScopeProvider;
