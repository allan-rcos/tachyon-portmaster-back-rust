//! Os parâmetros que uma listagem recebe da apresentação.
//!
//! Chegam como struct e não como argumentos soltos porque toda listagem tem os
//! mesmos três — cursor, limite, busca — mais o que é seu. Um `Option` a mais
//! numa assinatura de cinco argumentos passa despercebido; um campo novo numa
//! struct nomeada, não.

pub mod container_list_params;
pub mod list_params;
pub mod summary_list_params;
pub mod user_list_params;

pub use container_list_params::ContainerListParams;
pub use list_params::ListParams;
pub use summary_list_params::SummaryListParams;
pub use user_list_params::UserListParams;
