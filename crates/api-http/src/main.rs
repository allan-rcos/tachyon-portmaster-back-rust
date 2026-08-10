//! Entrada do servidor HTTP.
//!
//! Não há composition root: esta apresentação tem o seu próprio `main`, que lê os
//! segredos de runtime, chama `portmaster_app::register` — o qual encadeia
//! `domain` e `infra` — e sobe o axum sobre o router resultante.
//!
//! O `main` conhece **só o `app`**. Ele preenche `AppSecrets` com o que leu do
//! ambiente e recebe de volta algo que já sabe atender requisição; `domain` e
//! `infra` não aparecem nem como dependência do crate.
//!
//! ## Nada disto sobrevive ao boot
//!
//! O provider do `app` é consumido pelo `register` da apresentação, que o
//! destrincha em controllers; a `ApiConfig` é destrinchada nos valores que cada
//! construtor precisa. Depois que o router está montado, nem um nem outro existe
//! em memória — e por isso não há `Arc` segurando nada.

use portmaster_api_http::{config, router};
use portmaster_app::{Logger as _, SystemLogger};
use tokio::net::TcpListener;
use tokio::signal;

/// Sobe o processo: log, segredos, camadas, router e escuta.
///
/// Falta de segredo derruba o boot antes de qualquer conexão: um servidor que
/// sobe com o JWT num valor padrão aceita token forjado, e descobrir isso em
/// produção custa mais do que não subir.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging();
    SystemLogger::install();
    install_panic_hook();

    let secrets = config::secrets::Secrets::load()?;
    let address = format!("{}:{}", secrets.api.host, secrets.api.port);

    let app = portmaster_app::register(secrets.app).await?;
    let routes = router(app, secrets.api).await?;

    let listener = TcpListener::bind(&address).await?;
    SystemLogger::get().info("servidor no ar", [("address", &address)]);

    axum::serve(listener, routes)
        .with_graceful_shutdown(shutdown())
        .await?;

    SystemLogger::get().info("servidor encerrado", []);

    Ok(())
}

/// Última linha de defesa contra pânico.
///
/// A infra previne na fonte e o middleware `Recover` evita a queda; o que
/// escapar dos dois cai aqui, e o hook existe para que isso seja logado com o
/// máximo de contexto em vez de morrer em silêncio.
///
/// O pânico é reportado pelo logger de sistema **e** no stderr. O logger é o
/// caminho normal, e o stderr é o que sobra quando ele não é o caminho: um
/// pânico durante a inicialização do subscriber, ou durante o desligamento
/// depois que ele já foi baixado, não deixaria rastro nenhum.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        SystemLogger::get().error("pânico não capturado", [("panic", &info.to_string())]);

        #[allow(
            clippy::print_stderr,
            clippy::disallowed_macros,
            reason = "último recurso quando o próprio logger pode não estar de pé"
        )]
        {
            eprintln!("panic não capturado: {info}");
        }
    }));
}

/// Liga o log estruturado do processo.
///
/// É o **sink** por trás da abstração de logging: nada acima da `infra` emite
/// evento, mas alguém precisa dizer para onde as linhas vão, e esse alguém é o
/// processo.
///
/// JSON porque o destino é um coletor, não um humano com um terminal: o
/// `request_id` do span da requisição só serve para correlacionar se puder ser
/// consultado como campo. O formatador serializa os campos do span corrente por
/// padrão, e é isso que faz o id aparecer em toda linha da requisição — inclusive
/// nas que o `app` e a `infra` emitem lá no fundo.
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
///
/// Sem o handler de `SIGTERM` resta o Ctrl-C: é pior, mas não é motivo para o
/// servidor recusar-se a servir.
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
            Err(error) => {
                SystemLogger::get().warn(
                    "não foi possível ouvir SIGTERM",
                    [("error", &error.to_string())],
                );
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

    SystemLogger::get().info(
        "sinal de parada recebido; encerrando as requisições em andamento",
        [],
    );
}
