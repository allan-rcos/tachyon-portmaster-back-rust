//! Os elos da chain, um por grupo de configuração.
//!
//! Cada arquivo é um grupo: lê as variáveis dele, preenche o slot dele no
//! [`BootDraft`](crate::config::boot_draft::BootDraft), e se declara sozinho na
//! slice que o linker preenche. Nenhum elo conhece outro, e o
//! [`Secrets::load`](crate::config::secrets::Secrets::load()) não importa
//! nenhum.
//!
//! Antes disto havia uma função de noventa linhas lendo o ambiente inteiro de
//! uma vez — o tipo de função que só cresce. Um grupo novo agora é um arquivo
//! novo aqui e um campo no rascunho; nada existente é editado.

pub(crate) mod api_chain;
pub(crate) mod database_chain;
pub(crate) mod domain_chain;
pub(crate) mod jwt_chain;
