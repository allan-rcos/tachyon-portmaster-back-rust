//! A slice de elos que o linker preenche.
//!
//! A ordem em que os elos aparecem aqui **não é especificada** — é a ordem em
//! que o linker emitiu as seções. Isso não é problema, e a razão é a mesma que o
//! `DotEnvStarter` do PHP registra: cada elo preenche o próprio slot do rascunho
//! e não lê o de ninguém, então não há como um depender de outro ter rodado
//! antes.
//!
//! A regra que sustenta isso: **nenhum elo pode ler um slot que não é o dele**.
//! Um grupo cujo valor dependa de outro grupo não pertence a esta slice;
//! pertence ao [`BootDraft::into_secrets`](crate::config::boot_draft::BootDraft::into_secrets()),
//! que roda depois de todos e vê o rascunho inteiro.

use linkme::distributed_slice;

use crate::config::config_link::ConfigLink;

/// Os grupos de configuração que este binário lê do ambiente.
#[distributed_slice]
pub(crate) static CONFIG_LINKS: [ConfigLink];
