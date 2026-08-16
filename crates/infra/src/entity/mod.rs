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
//! Não há `*_row.rs`: cada entity deriva o `FromRow` do driver, e o que se
//! escreveria no corpo de um `from_row` é atributo do campo que o exige —
//! `deserialize_with` para o id, para o índice de enum e para o instante, `json`
//! para a coluna `JSON`. As conversões em si moram em
//! [`decode`](decode::Decode), uma vez cada.
//!
//! O ganho não é o tamanho: é que uma coluna nova passa a ser um campo, e não um
//! campo mais uma linha de leitura que pode ficar para trás.
//!
//! A exceção é o [`UserEntity`](user_entity::UserEntity), que tem um campo que
//! não é coluna: o derive não tem como pular um, e ele escreve o `FromRow`.
//!
//! A identidade é um [`EntityId`](entity_id::EntityId) e não dois campos:
//! guardar `id: String` e `raw_id: i64` lado a lado deixaria os dois para serem
//! mantidos em sincronia à mão em cada construtor.

pub(crate) mod codec;
pub(crate) mod container_entity;
pub(crate) mod decode;
pub(crate) mod entity_id;
pub(crate) mod manifest_cargo_entity;
pub(crate) mod product_entity;
pub(crate) mod role_entity;
pub(crate) mod user_entity;
