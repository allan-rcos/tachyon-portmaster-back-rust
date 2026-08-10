//! A conexão e a transação.
//!
//! Dois objetos, porque são dois tempos de vida. O **contexto** nasce e morre
//! com a tarefa e guarda a transação; o **handle** nasce no boot, guarda o pool
//! e é clonado para dentro de cada repositório. O contexto nunca precisa do
//! handle — é o handle que acha o contexto, pelo mapa da tarefa.
//!
//! É essa separação que dispensa um global: abrir a transação é a única
//! operação que precisa do pool, e ela acontece no handle, que o recebeu por
//! injeção. O que a slice do escopo constrói não precisa de nada em mãos.

pub(crate) mod intern;
pub(crate) mod mysql_transaction;
