//! O id ordenável, pelo xid.

use crate::id::SequentialIdGenerator;

/// Gerador de `request_id`, sobre xid.
#[derive(Clone, Copy)]
pub(crate) struct XidGenerator;

impl XidGenerator {
    /// Monta o gerador.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl SequentialIdGenerator for XidGenerator {
    fn next(&self) -> String {
        xid::new().to_string()
    }
}

#[cfg(test)]
#[path = "tests/xid_generator_test.rs"]
mod tests;
