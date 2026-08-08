//! Os read models: um por consulta, POD e por valor.
//!
//! Uma `View` é o formato que a leitura devolve — sem `dyn`, sem `Box`, sem
//! objeto de domínio. Guardar o tempo como `i64` de epoch em ms, o enum como
//! `i32` de índice e o id como `String` base62 faz a `View` atravessar até a
//! apresentação sem que nenhuma camada do meio precise reinterpretá-la.

pub mod account_view;
pub mod cargo_item_view;
pub mod container_list_view;
pub mod container_summary_list_view;
pub mod container_summary_view_item;
pub mod container_view_item;
pub mod metrics_view;
pub mod occupancy_view;
pub mod product_list_view;
pub mod product_view_item;
pub mod role_list_view;
pub mod role_view_item;
pub mod telemetry_log_view;
pub mod user_list_view;

pub use account_view::AccountView;
pub use cargo_item_view::CargoItemView;
pub use container_list_view::ContainerListView;
pub use container_summary_list_view::ContainerSummaryListView;
pub use container_summary_view_item::ContainerSummaryViewItem;
pub use container_view_item::ContainerViewItem;
pub use metrics_view::MetricsView;
pub use occupancy_view::OccupancyView;
pub use product_list_view::ProductListView;
pub use product_view_item::ProductViewItem;
pub use role_list_view::RoleListView;
pub use role_view_item::RoleViewItem;
pub use telemetry_log_view::TelemetryLogView;
pub use user_list_view::UserListView;
