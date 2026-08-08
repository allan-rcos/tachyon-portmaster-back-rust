//! Onde um contêiner está no seu ciclo de vida.

use std::fmt;

/// Onde um contêiner está no seu ciclo de vida no pátio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ContainerStatus {
    /// Registrado e sem carga.
    Empty = 0,
    /// Recebendo carga; ainda aceita embarque e desembarque.
    Loading = 1,
    /// Fechado para carga, aguardando despacho.
    Sealed = 2,
    /// Despachado. Estado final.
    InTransit = 3,
}

impl ContainerStatus {
    /// O índice gravado no banco e no fio.
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converte o índice de volta, recusando um valor que não corresponda a
    /// nenhuma variante em vez de escolher uma por aproximação.
    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Empty),
            1 => Some(Self::Loading),
            2 => Some(Self::Sealed),
            3 => Some(Self::InTransit),
            _ => None,
        }
    }
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Empty => "Empty",
            Self::Loading => "Loading",
            Self::Sealed => "Sealed",
            Self::InTransit => "InTransit",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Os índices das variantes são dado gravado, não detalhe do enum.
    ///
    /// Estes números estão em cada linha do banco. Se este teste quebrar depois
    /// de mexer no enum, o problema não é o teste: as linhas já existentes
    /// passaram a significar outra coisa.
    #[test]
    fn indices_das_variantes_sao_estaveis() {
        assert_eq!(ContainerStatus::Empty.as_i32(), 0);
        assert_eq!(ContainerStatus::Loading.as_i32(), 1);
        assert_eq!(ContainerStatus::Sealed.as_i32(), 2);
        assert_eq!(ContainerStatus::InTransit.as_i32(), 3);
    }

    #[test]
    fn indice_desconhecido_nao_vira_variante() {
        assert_eq!(ContainerStatus::from_i32(4), None);
        assert_eq!(ContainerStatus::from_i32(-1), None);
    }

    #[test]
    fn ida_e_volta_preserva_a_variante() {
        for status in [
            ContainerStatus::Empty,
            ContainerStatus::Loading,
            ContainerStatus::Sealed,
            ContainerStatus::InTransit,
        ] {
            assert_eq!(ContainerStatus::from_i32(status.as_i32()), Some(status));
        }
    }
}
