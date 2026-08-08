//! Onde a transação da requisição fica guardada.

use std::sync::Arc;

use sqlx::mysql::MySql;
use sqlx::Transaction;
use tokio::sync::Mutex;

/// O que o `task_local` do escopo guarda.
///
/// `Arc<Mutex<Option<..>>>` e não a transação nua: o `Arc` porque o task-local
/// entrega clones, o `Mutex` (assíncrono) porque duas chamadas concorrentes da
/// mesma requisição podem pedir a transação, e o `Option` porque o escopo nasce
/// vazio — abrir a transação é `begin`, não entrar no escopo.
pub(crate) type Slot = Arc<Mutex<Option<Transaction<'static, MySql>>>>;
