//! A consulta compilada e o que a monta.
//!
//! Do lado da **escrita** todo SQL é literal `const`. Do lado da **leitura** isso
//! não se sustenta: uma listagem tem filtro opcional, e o `WHERE` muda de forma
//! conforme o que chegou na querystring. Daí este construtor mínimo.
//!
//! ## Por que os binds moram em três listas
//!
//! Com placeholder posicional (`?`), a ordem em que os valores são ligados
//! **tem** que ser a ordem em que os `?` aparecem no texto. Guardar tudo numa
//! lista só faria a corretude depender de chamar os métodos na mesma sequência
//! em que as cláusulas são renderizadas — um acoplamento invisível, que quebra
//! calado no dia em que alguém mover uma linha.

pub mod sql_query;

pub(crate) mod bind;
pub(crate) mod select;

pub use sql_query::SqlQuery;

pub(crate) use bind::Bind;
pub(crate) use select::Select;
