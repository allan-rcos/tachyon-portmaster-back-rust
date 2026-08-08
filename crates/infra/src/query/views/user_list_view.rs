//! O read model `UserListView`.

use crate::query::views::AccountView;
use serde::{Deserialize, Serialize};

/// A listagem de usuários.
///
/// Sem cursor nem total: a listagem de usuários pagina por página/limite, não
/// por keyset, porque é a única consulta administrativa em que pular para uma
/// página arbitrária é o uso real.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserListView {
    /// Os usuários da página.
    pub items: Vec<AccountView>,
}
