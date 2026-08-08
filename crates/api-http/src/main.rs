//! Entrada do servidor HTTP.
//!
//! Não há composition root: esta apresentação tem o seu próprio `main`, que lê os
//! segredos de runtime, chama `portmaster_app::register` — o qual encadeia
//! `domain` e `infra` — e sobe o axum sobre o router resultante.
//!
//! O `main` conhece **só o `app`**. Ele preenche `AppSecrets` com o que leu do
//! ambiente e recebe de volta algo que já sabe atender requisição; `domain` e
//! `infra` não aparecem nem como dependência do crate.

use std::sync::Arc;

use portmaster_api_http::{config, router};
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Última linha de defesa contra pânico. A infra previne na fonte e o
    // middleware Recover evita a queda; o que escapar dos dois cai aqui, e o
    // hook existe para que isso seja logado com o máximo de contexto em vez de
    // morrer em silêncio.
    std::panic::set_hook(Box::new(|info| {
        tracing::error!(panic = %info, "pânico não capturado");
        // O `tracing` acima é o caminho normal, e o stderr é o que sobra quando
        // ele não é o caminho: um pânico durante a inicialização do subscriber,
        // ou durante o desligamento depois que ele já foi baixado, não deixaria
        // rastro nenhum. Duplicar aqui custa uma linha e cobre esse buraco.
        #[allow(
            clippy::print_stderr,
            clippy::disallowed_macros,
            reason = "último recurso quando o próprio tracing pode não estar de pé"
        )]
        {
            eprintln!("panic não capturado: {info}");
        }
    }));

    logging();

    // Falta de segredo derruba o boot antes de qualquer conexão: um servidor que
    // sobe com o JWT num valor padrão aceita token forjado, e descobrir isso em
    // produção custa mais do que não subir.
    let secrets = config::secrets::Secrets::load()?;
    let address = format!("{}:{}", secrets.api.host, secrets.api.port);

    let provider = Arc::new(portmaster_app::register(secrets.app).await?);
    let app = router(provider, secrets.api);

    let listener = TcpListener::bind(&address).await?;
    tracing::info!(address, "servidor no ar");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;

    tracing::info!("servidor encerrado");

    Ok(())
}

/// Liga o log estruturado do processo.
///
/// JSON porque o destino é um coletor, não um humano com um terminal: o
/// `request_id` que o middleware carimba só serve para correlacionar se puder
/// ser consultado como campo.
fn logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();
}

/// Espera o sinal de parada.
///
/// SIGTERM é o que o Docker manda ao parar um contêiner, e SIGINT é o Ctrl-C.
/// Atender aos dois é o que faz uma requisição em andamento terminar em vez de
/// virar conexão cortada no cliente durante um deploy.
async fn shutdown() {
    let interrupt = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            // Sem o handler resta o Ctrl-C: é pior, mas não é motivo para o
            // servidor recusar-se a servir.
            Err(error) => {
                tracing::warn!(error = %error, "não foi possível ouvir SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }

    tracing::info!("sinal de parada recebido; encerrando as requisições em andamento");
}
