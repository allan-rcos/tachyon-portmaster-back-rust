//! Um papel de domínio, montado para o teste.

use chrono::{DateTime, TimeZone as _, Utc};
use portmaster_domain::domain::Role;

/// Um papel que o teste controla por inteiro.
///
/// O `RoleModel` do `domain` é `pub(crate)` lá dentro — só o `RoleTM` o
/// constrói, que é justamente a regra que ele existe para sustentar. Como aqui o
/// table module é um mock, alguém precisa produzir o `Box<dyn Role>` que ele
/// devolve, e é este stub.
#[derive(Debug, Clone)]
pub(crate) struct StubRole {
    /// Identidade em base62.
    id: String,
    /// Nome do papel.
    name: String,
    /// Os slugs concedidos.
    permissions: Vec<String>,
}

impl StubRole {
    /// Um papel com este id e estas permissões.
    pub(crate) fn new(id: &str, permissions: &[&str]) -> Self {
        Self {
            id: id.to_owned(),
            name: "papel do teste".to_owned(),
            permissions: permissions.iter().map(|slug| (*slug).to_owned()).collect(),
        }
    }

    /// O mesmo papel dentro do `Box` que o table module devolveria.
    pub(crate) fn boxed(id: &str, permissions: &[&str]) -> Box<dyn Role> {
        Box::new(Self::new(id, permissions))
    }
}

/// O instante fixo das três datas.
///
/// Fixo porque nenhuma asserção depende delas: o que se testa aqui é a
/// orquestração, e uma data variável só faria o teste parecer sensível ao
/// relógio.
fn epoch() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("a data é válida e não é ambígua")
}

impl Role for StubRole {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn permissions(&self) -> &[String] {
        &self.permissions
    }

    fn created_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn updated_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn clone_role(&self) -> Box<dyn Role> {
        Box::new(self.clone())
    }
}
