//! O contexto de quem age, montado para o teste.

use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake as _;

use crate::context::{RoleContext, UserContext};

/// Um usuário que concede exatamente as permissões pedidas.
///
/// O que muda de um teste para outro é só a lista: identidade, nome e e-mail
/// vêm do `fake` porque nenhuma asserção depende deles, e fixá-los à mão faria
/// parecer que dependem.
pub(crate) fn user_with(permissions: &[&str]) -> UserContext {
    UserContext {
        id: "aZl8Y0".to_owned(),
        name: Name().fake(),
        email: SafeEmail().fake(),
        roles: vec![RoleContext {
            id: "bYk7X1".to_owned(),
            name: "papel do teste".to_owned(),
            permissions: permissions.iter().map(|slug| (*slug).to_owned()).collect(),
        }],
    }
}
