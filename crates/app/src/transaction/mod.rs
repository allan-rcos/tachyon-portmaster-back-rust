//! A moldura de transação que envolve um caso de uso de escrita.
//!
//! A regra é um export por arquivo, e o arquivo leva o nome do que exporta —
//! para o `Transaction`, isso dá `transaction/transaction.rs`.

#[allow(
    clippy::module_inception,
    reason = "o módulo `transaction` exporta o tipo `Transaction`: nome do arquivo = nome do tipo"
)]
pub(crate) mod transaction;
