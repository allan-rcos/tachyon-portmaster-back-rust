//! O usuário que acompanha uma sessão recém-aberta.

use crate::error::api_error::ApiError;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;

/// Monta o `User` que vai dentro do `LoginResponse`.
///
/// **É este arquivo que justifica a [`ResponseFactory`] ser tipada.** O
/// `LoginResponseFactory` embute o resultado desta aqui, e o campo do `.fbs`
/// espera um `User` — não bytes, não um offset apagado. Se a única trait de
/// resposta fosse a `Renderable`, o pai não teria como aninhar o filho.
pub(crate) struct UserResponseFactory {
    id: String,
    name: String,
    email: String,
}

impl UserResponseFactory {
    /// Monta a factory a partir do objeto de domínio.
    ///
    /// O que **não** atravessa é a garantia: um `User` de domínio tem
    /// `password_hash`, e a tabela do wire não tem onde pôr isso.
    pub(crate) fn of(user: &dyn portmaster_app::domain::User) -> Self {
        Self {
            id: user.id().to_owned(),
            name: user.name().to_owned(),
            email: user.email().to_owned(),
        }
    }
}

impl ResponseFactory for UserResponseFactory {
    type Table = fbs::auth::User;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::auth::User {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
        })
    }
}
