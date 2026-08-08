//! Product.

pub mod create_product_command;
pub mod delete_product_command;
pub mod update_product_command;

pub use create_product_command::CreateProductCommand;
pub use delete_product_command::DeleteProductCommand;
pub use update_product_command::UpdateProductCommand;
