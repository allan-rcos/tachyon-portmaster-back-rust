//! Hash de senha com Argon2id.
//!
//! Argon2id é o padrão recomendado hoje: além de custo de CPU, ele impõe custo
//! de **memória**, o que é o que arruína um ataque em GPU — onde milhares de
//! núcleos compensam bem a lentidão de CPU, mas não a falta de RAM por núcleo.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, SaltString};
use argon2::{Argon2, PasswordVerifier};

use crate::security::PasswordHasher;

/// Hasher de senha para armazenamento.
#[derive(Clone)]
pub struct Argon2Hasher {
    /// O Argon2 já configurado; construí-lo por chamada desperdiçaria o parse dos parâmetros.
    argon: Argon2<'static>,
}

impl Argon2Hasher {
    /// Monta o hasher com os parâmetros padrão do Argon2id.
    pub(crate) fn new() -> Self {
        Self {
            argon: Argon2::default(),
        }
    }
}

impl PasswordHasher for Argon2Hasher {
    /// Deriva o hash, com salt novo a cada chamada.
    ///
    /// O salt por chamada é o que faz duas contas com a mesma senha terem
    /// hashes diferentes, e o que inutiliza uma rainbow table.
    ///
    /// ## O que acontece quando o Argon2 falha
    ///
    /// Ele só falha por parâmetro inválido, e aqui os parâmetros são constantes
    /// — na prática, nunca. Ainda assim o caminho existe, e devolver string
    /// vazia é a saída segura: ela não é um hash PHC válido, então `verify`
    /// recusa qualquer senha contra ela. A conta fica inacessível em vez de
    /// aceitar qualquer um.
    fn hash(&self, plain: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);

        match self.argon.hash_password(plain.as_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(_) => String::new(),
        }
    }

    /// Confere a tentativa contra o hash guardado.
    ///
    /// Hash corrompido ou em formato desconhecido responde `false`: não há como
    /// conferir, e "não sei" tem que significar "não passa".
    fn verify(&self, plain: &str, hash: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(hash) else {
            return false;
        };

        self.argon
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok()
    }
}

#[cfg(test)]
#[path = "tests/argon2_hasher_test.rs"]
mod tests;
