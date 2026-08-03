//! Os middlewares, como classes `tower::Layer`.
//!
//! Cada um é uma struct construída a partir da configuração ou do provider e
//! aplicada com `.layer(...)`. A composição é **estática**: cada `.layer()`
//! embrulha o serviço num tipo novo, monomorfizado, sem `dyn` e sem
//! `middleware::from_fn`.
//!
//! ## Por que o futuro é boxeado, se a composição é estática
//!
//! O `Service::Future` é um tipo associado, e nomear o futuro de um `async
//! move` exigiria `type_alias_impl_trait` — que ainda é nightly. A saída no
//! estável é `BoxFuture`.
//!
//! O `dyn` está **no futuro de uma requisição**, não no grafo de serviços: a
//! stack continua sendo um tipo concreto conhecido em tempo de compilação, e o
//! custo é uma alocação por requisição — irrelevante ao lado do I/O que ela vai
//! fazer de qualquer forma. É o mesmo lugar onde a `infra` já admite `dyn` (a
//! borda de I/O), e não o wiring que a DI estática protege.
//!
//! ## A ordem importa
//!
//! Os `.layer()` do axum aplicam-se **de baixo para cima**: o último declarado é
//! o mais externo. A stack é montada em [`crate::router`], e a ordem lá está
//! comentada — trocá-la faz o `Recover` deixar de cobrir os outros, ou o
//! `Session` ficar indisponível para os handlers.

pub(crate) mod logging;
pub(crate) mod recover;
pub(crate) mod request_id;
pub(crate) mod timeout;
pub(crate) mod token;
