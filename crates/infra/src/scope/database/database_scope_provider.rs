//! Quem serve o acesso ao banco.

use std::sync::{PoisonError, RwLock};

use anyhow::Context as _;

use crate::config::InfraSecrets;
use crate::scope::database::intern::mariadb_unit_of_work::{
    mariadb_unit_of_work, MariaDbUnitOfWork,
};
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// O pool do processo.
///
/// Guardado porque não pode existir em dois lugares: um segundo pool dobraria
/// as conexões abertas, e o teto que o `POOL_MAX_CONNECTIONS` fixa deixaria de
/// significar o que diz.
///
/// `RwLock` porque quem o abre é configuração, e configuração se troca.
static UNIT_OF_WORK: RwLock<Option<MariaDbUnitOfWork>> = RwLock::new(None);

/// O acesso ao banco, para quem constrói um repositório.
pub(crate) struct DatabaseScopeProvider;

impl DatabaseScopeProvider {
    /// Abre o pool com estes segredos, e larga o que estava aberto.
    ///
    /// A troca não corta ninguém no meio: quem já segura um clone do handle
    /// antigo termina o que estava fazendo, e o pool antigo fecha as conexões
    /// quando o último clone dele cair.
    ///
    /// Abrir é preguiçoso — não toca a rede. Quem confirma que há um banco do
    /// outro lado é o [`Self::ping`], logo depois.
    pub(crate) fn install(secrets: &InfraSecrets) -> anyhow::Result<()> {
        let created = mariadb_unit_of_work(secrets)?;

        *UNIT_OF_WORK.write().unwrap_or_else(PoisonError::into_inner) = Some(created);

        Ok(())
    }

    /// A unidade de trabalho do processo.
    ///
    /// Sem pool aberto, isto falha em vez de inventar um padrão. É a diferença
    /// para a identidade do Snowflake, que tem um padrão que significa algo:
    /// não existe URI de banco padrão, e adivinhar uma só trocaria um erro
    /// claro no boot por um erro obscuro na primeira consulta.
    pub(crate) fn unit_of_work(
    ) -> anyhow::Result<impl MySqlTransaction + Sync + Clone + use<> + 'static> {
        Self::shared()
    }

    /// Confirma que o banco responde, e derruba o boot se não responder.
    ///
    /// Existe porque o pool nasce preguiçoso: até esta chamada nada tocou a
    /// rede, e um erro de credencial ainda não se manifestou.
    pub(crate) async fn ping() -> anyhow::Result<()> {
        Self::shared()?.ping().await
    }

    /// O pool, pelo tipo concreto que só este módulo enxerga.
    ///
    /// Separado do factory porque o [`Self::ping`] precisa do tipo, e não do
    /// contrato: `ping` não é uma operação da unidade de trabalho, é uma
    /// conferência de boot, e pô-la no trait a ofereceria a todo repositório.
    fn shared() -> anyhow::Result<MariaDbUnitOfWork> {
        UNIT_OF_WORK
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .context("o pool ainda não foi aberto: os segredos do banco precisam ser instalados antes de montar um repositório")
    }
}
