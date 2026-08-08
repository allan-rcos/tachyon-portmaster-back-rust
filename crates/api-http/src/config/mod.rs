//! Os segredos e knobs de runtime desta apresentação.
//!
//! Esta camada é a **única** que lê ambiente. Ela monta as structs de segredo de
//! todas as camadas — [`portmaster_app::AppSecrets`] chega pronta ao `register`,
//! que a distribui para dentro. As camadas internas nunca leem uma variável:
//! recebem o que precisam por argumento, e por isso são testáveis sem preparar
//! ambiente nenhum.
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
pub mod env;
pub mod jwt_config;
pub mod secrets;
