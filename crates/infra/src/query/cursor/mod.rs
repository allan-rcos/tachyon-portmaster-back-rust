//! A paginação por keyset.
//!
//! O cursor carrega o último id visto **e** os filtros sob os quais ele foi
//! emitido. Sem os filtros, mudar a busca no meio de uma paginação continuaria
//! de onde a anterior parou — pulando resultados em silêncio.

#[allow(
    clippy::module_inception,
    reason = "o módulo `cursor` exporta o tipo `Cursor`: é a regra de um export por arquivo, e o nome do arquivo é o do tipo"
)]
pub(crate) mod cursor;
pub(crate) mod cursor_filters;

pub(crate) use cursor::Cursor;
pub(crate) use cursor_filters::CursorFilters;
