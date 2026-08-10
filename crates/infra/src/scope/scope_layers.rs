//! A slice que o linker preenche.
//!
//! A ordem em que as camadas aparecem aqui **não é especificada** — é a ordem em
//! que o linker emitiu as seções. Um campo de prioridade para contorná-la seria
//! coordenação entre arquivos que não se conhecem: dois contextos novos
//! disputando o mesmo número, e o conflito de merge de volta.
//!
//! Daí a regra que a substitui: **nenhum contexto pode depender de outro ter
//! confirmado antes**. Ela se sustenta porque a invalidação do cache de view
//! acontece depois do escopo, no caso de uso — que é o único ponto onde a ordem
//! importaria, já que publicar cache antes de o SQL confirmar deixa dado
//! fantasma se o commit falhar. Um contexto futuro que precise confirmar depois
//! de outro não pertence a esta slice; pertence ao corpo do caso de uso.

use linkme::distributed_slice;

use crate::scope::scope_layer::ScopeLayer;

/// As camadas que participam de todo escopo aberto neste binário.
#[distributed_slice]
pub(crate) static SCOPE_LAYERS: [ScopeLayer];
