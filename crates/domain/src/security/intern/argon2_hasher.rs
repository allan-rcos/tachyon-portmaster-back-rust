//! Hash de senha com Argon2id.
//!
//! Argon2id é o padrão recomendado hoje: além de custo de CPU, ele impõe custo
//! de **memória**, o que é o que arruína um ataque em GPU — onde milhares de
//! núcleos compensam bem a lentidão de CPU, mas não a falta de RAM por núcleo.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, SaltString};
use argon2::{Argon2, PasswordVerifier};

use crate::security::PasswordHasher;

/// Monta o hasher com os parâmetros padrão do Argon2id.
///
/// Os parâmetros são do build, não de runtime: mudá-los invalidaria os hashes
/// já gravados, que carregam os seus próprios no formato PHC.
///
/// Montar é barato — ler constantes —, e dois deles não têm nada a
/// compartilhar. O que custa é derivar o hash, e isso é por chamada de
/// [`PasswordHasher::hash`].
pub(in crate::security) fn argon2_hasher(
) -> impl PasswordHasher + Send + Sync + Clone + use<> + 'static {
    Argon2Hasher {
        argon: Argon2::default(),
    }
}

/// Hasher de senha para armazenamento.
#[derive(Clone)]
struct Argon2Hasher {
    /// O Argon2 já configurado, com os parâmetros de custo do build.
    argon: Argon2<'static>,
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
