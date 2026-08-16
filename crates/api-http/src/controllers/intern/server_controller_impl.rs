//! O controller de estado do serviço. Não sai do módulo.

use crate::controllers::server_controller::ServerController;
use crate::wire::api_response::ApiResponse;
use crate::wire::vo::server::project_info_x::ProjectInfoX;

/// O nome com que o serviço se identifica.
const NAME: &str = "tachyon/portmaster";

/// Quantos bytes tem um MiB.
const BYTES_PER_MIB: f64 = 1024.0 * 1024.0;

/// Monta o controller de `/info`.
///
/// O nome do ambiente vem por argumento e não de um `static`: ele é o único
/// dado de config que um controller carrega, e é lido uma vez, no boot.
pub(crate) const fn server_controller(
    environment: String,
) -> impl ServerController + use<> + 'static {
    ServerControllerImpl { environment }
}

/// O handler de `/info`.
#[derive(Clone)]
struct ServerControllerImpl {
    /// Em que ambiente o processo está rodando.
    environment: String,
}

impl ServerController for ServerControllerImpl {
    #[allow(
        clippy::unused_async,
        reason = "assinatura de handler: as rotas são async mesmo quando não esperam nada"
    )]
    async fn info(self) -> ApiResponse<ProjectInfoX> {
        ApiResponse::ok(
            async {
                Ok(ProjectInfoX {
                    name: NAME.to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    environment: self.environment.clone(),
                    runtime: runtime(),
                    memory_usage_mb: resident_mib(),
                })
            }
            .await,
        )
    }
}

/// A versão do compilador que produziu este binário, com o runtime.
fn runtime() -> String {
    match env!("PORTMASTER_RUSTC_VERSION") {
        "" => "Rust + tokio/axum".to_owned(),
        version => format!("{version} + tokio/axum"),
    }
}

/// A memória residente do processo, em MiB.
///
/// Lê `/proc`, que só existe no Linux — o alvo de deploy. Onde não existir, o
/// campo sai zerado em vez de derrubar a rota: `/info` é o que alguém consulta
/// justamente quando algo está estranho.
fn resident_mib() -> f64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0.0;
    };

    resident_mib_of(&status)
}

/// Extrai o `VmRSS` de um `/proc/self/status`.
fn resident_mib_of(status: &str) -> f64 {
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        // O kernel escreve sempre em kB, e o sufixo faz parte do formato.
        .and_then(|value| value.split_whitespace().next())
        .and_then(|kib| kib.parse::<f64>().ok())
        .map(|kib| (kib * 1024.0 / BYTES_PER_MIB * 100.0).round() / 100.0)
        .unwrap_or_default()
}
