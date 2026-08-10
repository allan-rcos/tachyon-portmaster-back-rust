//! A implementação de domínio de um marcador.

use crate::domain::Marker;

/// A implementação do domínio de [`Marker`].
pub(crate) struct MarkerModel {
    /// O grupo do marcador — o namespace que separa um uso do outro.
    group: String,
    /// A chave dentro do grupo, já hasheada.
    key: String,
    /// O que o marcador afirma; `false` é a revogação.
    flag: bool,
}

impl MarkerModel {
    /// Monta um marcador a partir de campos já validados.
    ///
    /// `key` é o **digest** do valor, não o valor: quem chama já passou o texto
    /// pelo `IndexHasher`. O construtor não hasheia por conta própria porque
    /// isso o obrigaria a conhecer o hasher, e o model não conhece nada.
    pub(crate) const fn new(group: String, key: String, flag: bool) -> Self {
        Self { group, key, flag }
    }
}

impl Marker for MarkerModel {
    fn group(&self) -> &str {
        &self.group
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn flag(&self) -> bool {
        self.flag
    }
}
