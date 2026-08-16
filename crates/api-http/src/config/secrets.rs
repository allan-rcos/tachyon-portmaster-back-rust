//! Tudo que o processo precisa para subir.

use portmaster_app::AppSecrets;

use crate::config::api_config::ApiConfig;
use crate::config::boot_draft::BootDraft;
use crate::config::config_links::CONFIG_LINKS;
use crate::config::env_source::EnvSource;
use crate::config::jwt_config::JwtConfig;

/// Tudo que o processo precisa para subir, por grupo.
///
/// É o `BootConfig` do PHP: o agregado que atravessa a fronteira do boot uma vez
/// e é desmontado do outro lado. Manter os grupos separados é o que preserva o
/// isolamento — o `app` recebe só o [`AppSecrets`], e nada abaixo desta camada
/// enxerga uma [`ApiConfig`].
#[derive(Debug, Clone)]
pub struct Secrets {
    /// Onde o servidor escuta e como se comporta.
    pub api: ApiConfig,
    /// O token de sessão e os cookies que o carregam.
    pub jwt: JwtConfig,
    /// O que é das camadas de baixo.
    pub app: AppSecrets,
}

impl Secrets {
    /// Roda a chain sobre o ambiente e devolve a configuração completa.
    ///
    /// A chain existe só aqui dentro: os elos preenchem um rascunho, o rascunho
    /// vira esta struct, e nem um nem outro sobrevive à chamada. Nenhum objeto
    /// do sistema guarda um elo, e ninguém consulta o ambiente depois deste
    /// ponto.
    ///
    /// Ela roda **até o fim** mesmo com problemas pelo caminho. É o ponto do
    /// desenho: uma tentativa de boot é suficiente para ver a lista inteira do
    /// que está errado, em vez de corrigir uma variável, subir de novo, e
    /// descobrir a seguinte.
    ///
    /// Falta de segredo obrigatório derruba o boot. É deliberado: um sistema que
    /// sobe com o JWT num valor padrão aceita tokens que qualquer um pode
    /// forjar, e descobrir isso em produção é caro demais comparado a não subir.
    pub fn load() -> anyhow::Result<Self> {
        let mut env = EnvSource::of_process();
        let mut draft = BootDraft::default();

        for link in CONFIG_LINKS {
            (link.read)(&mut env, &mut draft);
        }

        env.into_result()?;

        draft.into_secrets()
    }
}
