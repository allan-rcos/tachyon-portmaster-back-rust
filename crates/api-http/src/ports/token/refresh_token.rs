//! O refresh token opaco.

/// O separador entre o id e a parte aleatória.
const SEPARATOR: char = '.';

/// O refresh token, que não é um JWT.
///
/// É opaco: `{id do usuário}.{aleatório}`. Não carrega claims, não é assinado, e
/// sozinho não afirma nada — sua validade é um marcador no cache, revogável a
/// qualquer momento. Um refresh-JWT seria válido até expirar mesmo depois do
/// logout, e é justamente isso que a rotação precisa impedir.
///
/// O id na frente não é segredo (é o id público do usuário); ele existe para que
/// a rotação saiba **quem** revalidar. A entropia toda está na segunda metade.
///
/// Struct-namespace e não um tipo com estado: não há o que guardar entre as duas
/// operações, e um `String` embrulhado só faria toda chamada desembrulhar.
pub struct RefreshToken;

impl RefreshToken {
    /// Monta um refresh token opaco para um usuário.
    pub fn issue(user_id: &str, random: &str) -> String {
        format!("{user_id}{SEPARATOR}{random}")
    }

    /// De quem é este refresh token.
    ///
    /// Só lê o prefixo — não afirma nada sobre validade. Quem decide se a marca
    /// ainda vale é o `MarkUseCase`, e quem decide se o usuário ainda existe é o
    /// `SessionUseCase`.
    pub fn owner_of(token: &str) -> Option<&str> {
        let (id, random) = token.split_once(SEPARATOR)?;

        (!id.is_empty() && !random.is_empty()).then_some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_refresh_diz_de_quem_e_sem_afirmar_validade() {
        let token = RefreshToken::issue("u1", "V1StGXR8Z5jdHi6BmyT");

        assert_eq!(RefreshToken::owner_of(&token), Some("u1"));
    }

    #[test]
    fn um_refresh_sem_as_duas_partes_nao_tem_dono() {
        // Recusar aqui evita consultar o banco por um id vazio.
        for malformado in ["", "só-id", ".aleatorio", "u1.", "."] {
            assert_eq!(
                RefreshToken::owner_of(malformado),
                None,
                "aceitou {malformado:?}"
            );
        }
    }
}
