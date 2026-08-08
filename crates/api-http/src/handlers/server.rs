//! `GET /info` — o que este processo é.
//!
//! A única rota **pública** com corpo: responde antes de existir um usuário no
//! sistema, e é como um operador confirma que apontou para o servidor certo
//! antes de rodar o `/setup`. Não há nada aqui que descreva topologia — nome,
//! versão, ambiente e uso de memória — e por isso ela pode ser pública.

use crate::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::dto::server::project_info_factory::ProjectInfoFactory;
use crate::wire::wire::Wire;

/// O nome do projeto, como o PHP o publicava.
const NAME: &str = "tachyon/portmaster";

/// Quantos bytes num mebibyte.
const BYTES_PER_MIB: f64 = 1024.0 * 1024.0;

/// Os handlers de introspecção.
///
/// Diferente dos demais: não tem caso de uso nenhum atrás, porque o que ele
/// responde é sobre o próprio processo. Nada disto passa pelo `app`.
pub struct ServerHandlers {
    environment: String,
}

impl ServerHandlers {
    /// Monta os handlers com o nome do ambiente.
    pub(crate) const fn new(environment: String) -> Self {
        Self { environment }
    }

    /// `GET /info`
    #[allow(
        clippy::unused_async,
        reason = "assinatura de handler do axum: as rotas são async mesmo quando não esperam nada"
    )]
    pub(crate) async fn info(&self, wire: Wire) -> Result<ApiResponse, ApiError> {
        Ok(ApiResponse::ok(
            wire,
            ProjectInfoFactory::of(
                NAME,
                env!("CARGO_PKG_VERSION"),
                self.environment.clone(),
                runtime(),
                resident_mib(),
            ),
        ))
    }
}

/// O que está executando, como o `/info` o publica.
///
/// A versão do compilador vem do `build.rs`; um binário construído sem ela
/// responde só "Rust", que é menos informação e não uma informação errada.
fn runtime() -> String {
    match env!("PORTMASTER_RUSTC_VERSION") {
        "" => "Rust + tokio/axum".to_owned(),
        version => format!("{version} + tokio/axum"),
    }
}

/// A memória residente do processo, em MiB.
///
/// Lida do `/proc`, que é onde o número existe sem custar uma dependência. Fora
/// do Linux — ou se o arquivo mudar de forma — responde `0`: o campo é
/// informativo, e derrubar o `/info` por não saber quanta memória se usa
/// tornaria inútil a rota que serve justamente para dizer que o servidor está de
/// pé.
fn resident_mib() -> f64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0.0;
    };

    resident_mib_of(&status)
}

/// Extrai o `VmRSS` de um `/proc/self/status`.
///
/// Separado da leitura para poder ser testado sem depender do processo real.
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn le_o_residente_em_mib_com_duas_casas() {
        let status = "Name:\tportmaster\nVmSize:\t 123456 kB\nVmRSS:\t   2048 kB\n";

        assert_eq!(resident_mib_of(status), 2.0);
    }

    #[test]
    fn um_status_sem_o_campo_nao_derruba_a_rota() {
        // `/info` existe para dizer que o servidor está de pé; falhar nele por
        // não saber a memória seria trocar a resposta pela pergunta.
        assert_eq!(resident_mib_of("Name:\tportmaster\n"), 0.0);
        assert_eq!(resident_mib_of(""), 0.0);
    }

    #[test]
    fn um_valor_ilegivel_conta_como_ausente() {
        assert_eq!(resident_mib_of("VmRSS:\tmuita kB\n"), 0.0);
    }
}
