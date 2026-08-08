//! O conjunto de filtros sob os quais um cursor vale.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Os filtros sob os quais um cursor foi emitido.
///
/// `BTreeMap` por dentro, e não `HashMap`: a serialização precisa ser estável
/// para que o mesmo conjunto de filtros produza sempre o mesmo JSON. Com ordem
/// de iteração aleatória, dois cursores idênticos poderiam não se reconhecer.
///
/// É um tipo próprio e não um alias porque o construtor pertence a ele — no
/// molde do `Base62`, o nome do tipo é o namespace da operação. `#[serde(transparent)]`
/// mantém o formato do token idêntico ao que o alias produzia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct CursorFilters(BTreeMap<String, String>);

impl CursorFilters {
    /// Monta o mapa de filtros a partir de pares nomeados.
    ///
    /// Os valores viram texto porque o mapa existe só para ser **comparado**
    /// inteiro, nunca lido de volta como número. Um `None` entra como string
    /// vazia para que "sem filtro" seja um valor tão explícito quanto qualquer
    /// outro — se a chave sumisse, dois filtros diferentes poderiam gerar o
    /// mesmo mapa.
    pub(crate) fn of<const N: usize>(pairs: [(&str, String); N]) -> Self {
        Self(
            pairs
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }
}
