//! A fronteira interna da apresentação: o `router` pede aqui.

use crate::config::jwt_config::JwtConfig;
use crate::controllers::account_controller::AccountController;
use crate::controllers::auth_controller::AuthController;
use crate::controllers::container_controller::ContainerController;
use crate::controllers::manifest_controller::ManifestController;
use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::metrics_controller::MetricsController;
use crate::controllers::product_controller::ProductController;
use crate::controllers::role_controller::RoleController;
use crate::controllers::server_controller::ServerController;
use crate::controllers::user_controller::UserController;
use crate::controllers::ControllersProvider;
use crate::ports::token::TokenProvider;

/// Os factories da apresentação.
///
/// A configuração desta camada não é campo de ninguém: cada valor é
/// **instalado** no boot em quem o consome, e tem o seu método — o nome do
/// ambiente por [`Self::install_environment`], o segredo do token por
/// [`Self::install_jwt`].
///
/// Os controllers são construídos uma vez, aqui, e clonados por requisição.
///
/// Oito dos dez devolvem `Result`, e a divisão atravessa as camadas desde a
/// `infra`: quem depende do pool pode falhar antes de o boot ter instalado os
/// segredos do banco, e quem só toca memória ou config não tem como.
pub(crate) struct ApiProvider;

impl ApiProvider {
    /// Instala o nome do ambiente que o `/info` responde.
    ///
    /// Instalar de novo troca, e muda o que o `/info` responde da próxima
    /// requisição em diante.
    pub(crate) fn install_environment(environment: String) {
        ControllersProvider::install_environment(environment);
    }

    /// Instala o segredo que assina e confere o access token.
    ///
    /// Separado do ambiente de propósito: são duas configurações com donos
    /// diferentes e consequências diferentes, e juntá-las num `install` só
    /// obrigaria quem quer trocar uma a ter a outra em mãos. Instalar de novo é
    /// rotação de chave — invalida todo token já emitido.
    pub(crate) fn install_jwt(jwt: &JwtConfig) {
        TokenProvider::install(jwt);
    }

    /// A conta do próprio usuário.
    pub(crate) fn account_controller() -> anyhow::Result<impl AccountController + use<> + 'static> {
        ControllersProvider::account()
    }

    /// Sessão: login, setup, refresh e logout.
    pub(crate) fn auth_controller() -> anyhow::Result<impl AuthController + use<> + 'static> {
        ControllersProvider::auth()
    }

    /// Contêineres.
    pub(crate) fn container_controller(
    ) -> anyhow::Result<impl ContainerController + use<> + 'static> {
        ControllersProvider::container()
    }

    /// Carga e telemetria.
    pub(crate) fn manifest_controller() -> anyhow::Result<impl ManifestController + use<> + 'static>
    {
        ControllersProvider::manifest()
    }

    /// Metadados de sistema.
    pub(crate) fn metadata_controller() -> impl MetadataController + use<> + 'static {
        ControllersProvider::metadata()
    }

    /// O painel do pátio.
    pub(crate) fn metrics_controller() -> anyhow::Result<impl MetricsController + use<> + 'static> {
        ControllersProvider::metrics()
    }

    /// Produtos.
    pub(crate) fn product_controller() -> anyhow::Result<impl ProductController + use<> + 'static> {
        ControllersProvider::product()
    }

    /// Papéis.
    pub(crate) fn role_controller() -> anyhow::Result<impl RoleController + use<> + 'static> {
        ControllersProvider::role()
    }

    /// O estado do próprio processo.
    pub(crate) fn server_controller() -> impl ServerController + use<> + 'static {
        ControllersProvider::server()
    }

    /// Usuários.
    pub(crate) fn user_controller() -> anyhow::Result<impl UserController + use<> + 'static> {
        ControllersProvider::user()
    }
}
