//! O id ordenável, pelo xid.

use crate::id::SortableIdGenerator;

/// Gerador de `request_id`, sobre xid.
#[derive(Clone, Copy)]
pub(crate) struct XidGenerator;

impl XidGenerator {
    /// Monta o gerador.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl SortableIdGenerator for XidGenerator {
    fn next(&self) -> String {
        xid::new().to_string()
    }
}
