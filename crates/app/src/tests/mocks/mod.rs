//! Os mocks das ports que os services consomem.
//!
//! Um por arquivo, como todo o resto do repositório. O `mockall` exige que o
//! `mock!` cite `trait_variant::make` pelo nome canônico e **antes** do corpo do
//! `impl`, que é o que faz o mock nascer com a mesma variante `Send` que a trait
//! de produção tem. Os table modules são síncronos e não levam o atributo.

pub(crate) mod auth_tm_mock;
pub(crate) mod container_repository_mock;
pub(crate) mod container_tm_mock;
pub(crate) mod manifest_repository_mock;
pub(crate) mod manifest_tm_mock;
pub(crate) mod marker_group_repository_mock;
pub(crate) mod marker_group_tm_mock;
pub(crate) mod marker_repository_mock;
pub(crate) mod marker_tm_mock;
pub(crate) mod permission_repository_mock;
pub(crate) mod permission_tm_mock;
pub(crate) mod product_repository_mock;
pub(crate) mod product_tm_mock;
pub(crate) mod query_repository_stub;
pub(crate) mod role_repository_mock;
pub(crate) mod role_tm_mock;
pub(crate) mod user_repository_mock;
pub(crate) mod user_tm_mock;
pub(crate) mod view_cache_repository_mock;
