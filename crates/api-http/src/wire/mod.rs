//! O formato de fio, e os dois processos que o produzem.
//!
//! Uma requisição chega em `FlatBuffers` ou JSON e uma resposta sai em um dos
//! dois, negociado por `Content-Type`/`Accept`. Traduzir entre esses formatos
//! são na verdade **duas responsabilidades diferentes**, e mantê-las separadas é
//! o que permite acrescentar um terceiro formato sem reescrever nada:
//!
//! * **Serializar/desserializar** ([`strategy`]) não depende de *qual* objeto
//!   está em jogo. Todo tipo `FlatBuffers` desserializa do mesmo jeito, e todo
//!   tipo serde também. Cada strategy conhece um formato e nada sobre o payload.
//! * **Construir** ([`factory`]) depende dos dados e não do formato. O handler
//!   sabe o que responder, mas não sabe o que foi negociado — então entrega os
//!   dados a uma factory, que a strategy conduz. Todas devolvem a mesma coisa: o
//!   corpo da resposta.
//!
//! ## Onde está o único `dyn`
//!
//! No [`Wire`](wire::Wire), que carrega um `Arc<dyn EncodeStrategy>`. É o preço
//! de a saída ter que escolher em tempo de execução, e ele é cobrado uma vez: o
//! `Arc` nasce no boot (as strategies são ZSTs) e a requisição clona o ponteiro.
//!
//! A entrada não paga nada. `DecodeStrategy::decode` é genérico sobre a factory
//! e por isso a trait **não** é object-safe — um `match` no
//! [`Body`](body::Body) resolve a escolha sem vTable, e o caminho de requisição
//! continua 100% monomorfizado.
//!
//! Os schemas `.fbs` são a fonte compartilhada com o cliente e não são alterados
//! por nada disto. Os tipos `FlatBuffers` em [`fbs`] são gerados a partir deles
//! no build.

pub(crate) mod api_response;
pub(crate) mod body;
pub(crate) mod convert;
pub(crate) mod dto;
pub(crate) mod factory;
pub(crate) mod json;
pub(crate) mod json_body;
pub(crate) mod media_type;
pub(crate) mod no_content;
pub(crate) mod strategy;
#[allow(
    clippy::module_inception,
    reason = "o módulo `wire` exporta o tipo `Wire`: nome do arquivo = nome do tipo"
)]
pub(crate) mod wire;

/// Tipos `FlatBuffers` gerados pelo planus a partir dos schemas, no build.
///
/// O módulo é gerado inteiro — não editar aqui, e sim no `.fbs` correspondente.
///
/// `#[doc(hidden)]` porque o `rustdoc` não tem lista de exclusão: o que o
/// `phpdoc.dist.xml` do PHP resolve com um `<ignore>`, aqui só o atributo
/// resolve. Sem ele são 135 páginas de tabela gerada no render, reescritas a
/// cada mudança de schema e descrevendo o que ninguém chama direto — o que a
/// base de código usa são os DTOs e factories em [`dto`], que ficam.
#[doc(hidden)]
#[allow(
    dead_code,
    unsafe_code,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction,
    clippy::allow_attributes_without_reason,
    unused_imports,
    missing_docs,
    rustdoc::all
)]
pub mod fbs {
    include!(concat!(env!("OUT_DIR"), "/wire_generated.rs"));
}

/// As tabelas do wire, sem o aninhamento de namespace do gerador.
///
/// O planus reproduz `API.Fbs.Account` como `api::fbs::account`, o que faria
/// toda factory escrever `fbs::api::fbs::account::…`. O apelido encurta isso
/// para `tables::account::…` sem esconder de onde vem.
pub use fbs::api::fbs as tables;
