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

pub(crate) mod codec;
pub(crate) mod container_entity;
pub(crate) mod container_row;
pub(crate) mod manifest_cargo_entity;
pub(crate) mod manifest_cargo_row;
pub(crate) mod product_entity;
pub(crate) mod product_row;
pub(crate) mod role_entity;
pub(crate) mod role_row;
pub(crate) mod user_entity;
pub(crate) mod user_row;
