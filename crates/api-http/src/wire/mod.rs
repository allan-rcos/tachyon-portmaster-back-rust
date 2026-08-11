//! O formato de fio, e os quatro objetos que o produzem.
//!
//! Uma requisição chega em `FlatBuffers` ou JSON e uma resposta sai em um dos
//! dois, negociado por `Content-Type`/`Accept`. Quatro camadas resolvem isso, e
//! mantê-las separadas é o que permite acrescentar um terceiro formato sem
//! reescrever nada:
//!
//! * O **VO** ([`vo`]) é a mensagem, independente de formato. É o único destes
//!   quatro que um controller conhece.
//! * Os **DTOs** são dois, um por formato: a tabela que o planus gera do `.fbs`
//!   e a struct de [`dto::json`] com `#[derive(Serialize)]`. São **objetos
//!   diferentes** de propósito — colar os dois amarraria o corpo textual à forma
//!   do schema binário.
//! * As **traits de [`x`]** ligam o VO aos seus dois DTOs.
//! * As **strategies** ([`strategy`]) serializam, uma por formato.
//!
//! ## O contexto do Strategy não mora aqui
//!
//! Ele é o escopo da requisição, em `middleware/intern/{encode,decode}_context`,
//! e o que o resto do sistema vê são as portas de encode e decode. Havia aqui um
//! `Encoder` e um `Decoder` que faziam esse papel — o `Encoder` era um extractor
//! que trinta assinaturas de rota declaravam só para repassá-lo ao construtor da
//! resposta, e o `Decoder` era remontado dentro do extractor de corpo a cada
//! requisição. Negociar virou uma decisão só, tomada uma vez por requisição, por
//! um middleware.
//!
//! ## Não há `dyn` em lugar nenhum
//!
//! `EncodeStrategy::encode` e `DecodeStrategy::decode` são genéricos sobre o VO,
//! e por isso as traits não são object-safe — o que é o ponto, não um efeito
//! colateral. O que varia em tempo de execução é **qual** strategy está em uso, e
//! isso é a variante de [`media_type::MediaType`] guardada no escopo: um `match`
//! monomorfizado, sem vTable, sem `Arc`, sem alocação.
//!
//! A negociação também não atravessa a aplicação. O controller devolve um VO e
//! não sabe que ela existe.
//!
//! Os schemas `.fbs` são a fonte compartilhada com o cliente e não são alterados
//! por nada disto. Os tipos `FlatBuffers` em [`fbs`] são gerados a partir deles
//! no build.

pub(crate) mod api_response;
pub(crate) mod body;
pub(crate) mod dto;
pub(crate) mod media_type;
pub(crate) mod strategy;
pub(crate) mod vo;
pub(crate) mod x;

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
