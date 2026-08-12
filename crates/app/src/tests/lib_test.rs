//! Os testes de `lib`.

use crate::commands::session::LoginCommand;
use crate::error::{ProductError, SessionError};
use crate::queries::product::ListProductsQuery;
use crate::services::{ProductService, SessionService};
use crate::*;
use std::sync::Arc;

/// Reproduz o que o `api-http` fará com o provider.
///
/// É o teste que mais importa nesta camada, e ele não roda nada: se compila,
/// passou. A DI é 100% estática, então `Send` do futuro de um caso de uso é
/// **inferido** do tipo concreto — e a inferência só acontece porque tudo
/// aqui é genérico, sem `dyn`. Se um port passasse a segurar algo `!Send`
/// através de um `.await`, todo handler do axum deixaria de compilar, e o
/// erro apareceria lá — a três camadas de distância da causa.
///
/// Ver tmp/architecture/lifetimes-auto-traits.md: o bound de `Send` mora no
/// ponto de uso, que é exatamente isto.
#[allow(dead_code, reason = "o teste é a compilação: nunca é chamado")]
/// O `tokio::spawn` no corpo é o que prova o ponto: ele exige
/// `Send + 'static`, ou seja, que o futuro do caso de uso atravesse uma
/// fronteira de execução como atravessaria num handler.
async fn o_provider_serve_um_handler_do_axum<P>(provider: Arc<P>) -> Result<(), ProductError>
where
    P: AppProvider + Send + Sync + 'static,
{
    let context = context::UserContext {
        id: "1".into(),
        name: "Ana".into(),
        email: "ana@portmaster.local".into(),
        roles: Vec::new(),
    };

    let handle = tokio::spawn(async move {
        provider
            .product_service()
            .list(ListProductsQuery {
                context,
                cursor: None,
                limit: None,
                search: None,
            })
            .await
    });

    handle.await.expect("a tarefa não deve entrar em pânico")?;

    Ok(())
}

/// O mesmo, para um caso de uso de escrita — que devolve `Box<dyn …>`.
///
/// A exceção de borda do `dyn` não pode custar a `Send`-ness do retorno,
/// senão o objeto de domínio não atravessaria até a apresentação.
#[allow(dead_code, reason = "o teste é a compilação: nunca é chamado")]
async fn a_escrita_tambem_atravessa_uma_tarefa<P>(provider: Arc<P>) -> Result<(), SessionError>
where
    P: AppProvider + Send + Sync + 'static,
{
    let handle = tokio::spawn(async move {
        provider
            .session_service()
            .login(LoginCommand {
                email: "ana@portmaster.local".into(),
                password: "Portmaster1".into(),
            })
            .await
    });

    let user = handle.await.expect("a tarefa não deve entrar em pânico")?;

    // O trait reexportado é o que a apresentação usa para mapear ao fio.
    let _: &str = user.id();

    Ok(())
}
