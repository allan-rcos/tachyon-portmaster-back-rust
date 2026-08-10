//! O armazenamento em memória, e o que ele deve à tarefa.
//!
//! Mesma divisão do banco, pela mesma razão: o **contexto** guarda o que a
//! tarefa ainda não publicou, e o **handle** guarda o cache. O handle é injetado
//! nos repositórios; o contexto nasce com o escopo.
//!
//! ## Por que há mais de um store, e não um mapa só
//!
//! Cada store tem a sua política de despejo, e misturá-las seria um bug de
//! autorização esperando acontecer: o catálogo de permissões é preenchido no
//! boot e precisa viver enquanto o processo viver, enquanto o cache de view
//! existe **para** ser despejado. Num mapa único, uma rajada de listagens
//! despejaria permissões, e a próxima checagem de autorização recusaria em
//! silêncio quem tinha direito.
//!
//! O grupo dentro de cada store separa os recortes; o store separa as políticas.

pub(crate) mod intern;
pub(crate) mod memory_store;
