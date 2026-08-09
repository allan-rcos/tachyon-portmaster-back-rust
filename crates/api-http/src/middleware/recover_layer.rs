//! O layer de captura de pânico.

use tower::Layer;

use super::recover::Recover;

/// Aplica o [`Recover`].
#[derive(Clone, Copy, Default)]
pub(crate) struct RecoverLayer;

impl RecoverLayer {
    /// Monta o layer.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RecoverLayer {
    type Service = Recover<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Recover { inner }
    }
}
