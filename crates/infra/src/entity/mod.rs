//! As entities: onde o domínio encosta na tabela.
//!
//! Cada entity implementa o trait de domínio correspondente **e** concentra tudo
//! que é de banco — nome de tabela, nome de coluna, tipo de cada campo. Isso
//! mantém o model do `domain` limpo de qualquer vestígio de persistência, e é o
//! que permitiria trocar de banco mexendo só aqui.
//!
//! Como o trait de domínio é somente-leitura, uma entity **não consegue editar**
//! um objeto vindo de outra camada: ela o **recria** a partir do trait, por um
//! `from_domain`. A ausência de setters não é inconveniente, é a garantia.
//!
//! ## A entity **é** a linha
//!
//! Não há `*_row.rs`, e também não há `from_row` escrito à mão: cada entity
//! deriva `sqlx::FromRow`, e o que antes se escrevia no corpo daquela função
//! virou atributo do campo que o exige — `try_from` para o id e para o índice de
//! enum, `json` para a coluna `JSON`, `default` para o campo que não é coluna.
//!
//! O ganho não é o tamanho: é que uma coluna nova passa a ser um campo, e não um
//! campo mais uma linha de leitura que pode ficar para trás.
//!
//! A identidade é um [`EntityId`](entity_id::EntityId) e não dois campos. Antes
//! cada entity guardava `id: String` e `raw_id: i64` lado a lado, mantidos em
//! sincronia à mão nos dois construtores.

pub(crate) mod codec;
pub(crate) mod container_entity;
pub(crate) mod entity_id;
pub(crate) mod manifest_cargo_entity;
pub(crate) mod product_entity;
pub(crate) mod role_entity;
pub(crate) mod user_entity;
