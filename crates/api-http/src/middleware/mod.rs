//! O contexto da requisição: as portas na raiz, as implementações em `intern`.
//!
//! ## Quem usa o quê
//!
//! **Abrir** um escopo é assunto de middleware, e **consumir** o que há dentro
//! dele é, em geral, assunto de controller. A divisão do diretório é essa e nada
//! mais: os `*_port.rs` daqui são o que um controller injeta, e [`intern`] guarda
//! os layers que abrem os escopos e os contextos que os atendem.
//!
//! A exceção é o escopo cujo consumidor também é middleware — a pilha de eventos,
//! onde um layer abre e outro lê para carimbar a resposta. Ali não há porta neste
//! diretório, porque não há controller no caminho: o contrato vem do `app`, que é
//! de quem escreve na pilha.
//!
//! É o que fecha a porta para o abuso óbvio. O escritor de cada contexto é
//! `pub(super)`, alcançado só pelo layer irmão — de fora de `intern` não há
//! como acrescentar um usuário à sessão sem passar pelo middleware de token,
//! nem carimbar um `request_id` diferente do que a requisição recebeu.
//!
//! ## Cada middleware é um arquivo
//!
//! O `tower::Service` que faz o trabalho e o `tower::Layer` que o aplica moram
//! juntos, num tipo só: o `Service` nunca é nomeado de fora, porque quem monta a
//! pilha aplica o `Layer`. A pilha é composta por generics — cada
//! `.layer()` embrulha o serviço num tipo novo —, então não há `dyn` nem
//! `middleware::from_fn` solto em ponto nenhum dela.

pub(crate) mod cookie_port;
pub(crate) mod decode_port;
pub(crate) mod encode_port;
pub(crate) mod request_id_port;
pub(crate) mod session_port;

pub(crate) mod middleware_provider;

pub(crate) mod intern;

pub(crate) use middleware_provider::MiddlewareProvider;
