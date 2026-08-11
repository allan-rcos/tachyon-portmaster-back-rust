//! Os segredos e knobs de runtime desta apresentação.
//!
//! Esta camada é a **única** que lê ambiente. Ela monta as structs de segredo de
//! todas as camadas — [`portmaster_app::AppSecrets`] chega pronta ao `register`,
//! que a distribui para dentro. As camadas internas nunca leem uma variável:
//! recebem o que precisam por argumento, e por isso são testáveis sem preparar
//! ambiente nenhum.
//!
//! ## Uma chain, um grupo por elo
//!
//! A leitura não é uma função só. Cada grupo de configuração é um elo em
//! [`chain`], que se declara sozinho na slice que o linker preenche e escreve no
//! slot dele do [`boot_draft::BootDraft`]. O
//! [`secrets::Secrets::load()`] percorre a slice, congela o rascunho e descarta
//! tudo — a chain não sobrevive ao boot.
//!
//! É a Chain of Responsibility do `DotEnvChain` do PHP, pela mesma razão que ela
//! existe lá: a alternativa era uma classe com um campo, um setter e um ramo de
//! validação por variável, que só cresce.
//!
//! ## O que é runtime e o que é build
//!
//! Aqui só entra **segredo ou identidade de deploy**: senha do banco, segredo do
//! JWT, host/porta, quem é este servidor na composição do Snowflake. Knobs de
//! **arquitetura** — tamanho de pool, capacidade e TTL dos caches, estratégia de
//! id — são features de compilação, e não têm variável correspondente de
//! propósito: um `if` em produção sobre uma decisão de arquitetura é um bug
//! esperando o dia de errar.

pub mod api_config;
pub mod jwt_config;
pub mod secrets;

pub(crate) mod chain;

pub(crate) mod boot_draft;
pub(crate) mod config_link;
pub(crate) mod config_links;
pub(crate) mod env;
pub(crate) mod env_source;
