//! A abertura do pool de conexões.
//!
//! Compartilhado por clone — o `Pool` é um ponteiro contado por dentro, então
//! cada repositório carrega uma referência, não um pool próprio. É recurso de
//! borda: único, externalizado, e impossível de monomorfizar por consumidor.
//!
//! Quem chama é o construtor da
//! [`MariaDbUnitOfWork`](super::mariadb_unit_of_work::MariaDbUnitOfWork), e não
//! o provider direto: o pool não tem uso fora do handle que o carrega, e um
//! `open()` solto convidaria a que tivesse.
//!
//! Abrir não conecta. A primeira conexão sai da primeira consulta, ou do `ping`
//! que o boot faz de propósito para não subir com um banco inalcançável.

use std::path::PathBuf;

use anyhow::{bail, Context};
use mysql_async::{Opts, OptsBuilder, Pool, PoolConstraints, PoolOpts, SslOpts};
use secrecy::ExposeSecret;

use crate::config::pool::POOL_MAX_CONNECTIONS;
use crate::config::{DatabaseSslMode, InfraSecrets};

/// Abre o pool sem tocar a rede.
///
/// `Pool::new` não conecta: monta o pool e deixa a primeira conexão para a
/// primeira consulta. É o que mantém esta função **síncrona**, e com ela toda a
/// cadeia de factories acima — um handshake TCP aqui faria o `async` subir até o
/// router.
///
/// Quem toca a rede é o
/// [`ping`](super::mariadb_unit_of_work::MariaDbUnitOfWork::ping), passo
/// explícito do boot: sem ele um erro de credencial só apareceria na primeira
/// requisição, com o processo já reportado como saudável.
///
/// ## O fuso da sessão
///
/// O `init` roda em toda conexão nova do pool e fixa a sessão em UTC. Nenhuma
/// consulta depende disso hoje: não há coluna de data — o tempo é `BIGINT` com
/// epoch em milissegundos — e nenhuma função de tempo do servidor aparece no
/// SQL. É rede, não mecanismo: o dia em que alguém escrever um `NOW()` num
/// `SELECT`, ele responde em UTC como o resto da aplicação.
///
/// Vem depois do parse da URI de propósito: uma `?timezone=` na URI o
/// sobrescreveria sem que nada avisasse.
pub(super) fn open(secrets: &InfraSecrets) -> anyhow::Result<Pool> {
    let parsed = Opts::from_url(secrets.database_uri.expose_secret())
        .context("a URI do banco não está num formato que o driver entenda")?;

    let opts = OptsBuilder::from_opts(parsed)
        .init(vec!["SET time_zone = '+00:00'"])
        .ssl_opts(ssl_opts(secrets)?)
        .pool_opts(pool_opts()?);

    Ok(Pool::new(opts))
}

/// Fixa o teto de conexões simultâneas.
///
/// O piso é zero, e não o padrão do driver: uma instalação pequena não deve
/// segurar dez conexões ociosas contra um servidor que talvez nem as ofereça. O
/// teto é decisão de arquitetura, escolhida por feature de compilação — ver
/// [`POOL_MAX_CONNECTIONS`].
fn pool_opts() -> anyhow::Result<PoolOpts> {
    let max = usize::try_from(POOL_MAX_CONNECTIONS)
        .context("o teto de conexões do pool não cabe nesta plataforma")?;

    let constraints = PoolConstraints::new(0, max)
        .with_context(|| format!("{max} não é um teto de conexões utilizável"))?;

    Ok(PoolOpts::default().with_constraints(constraints))
}

/// Traduz o modo de TLS dos segredos nas opções do driver.
///
/// `verify_ca` sem o bundle da CA é recusado aqui: sem a cadeia não há o que
/// validar, e cair em silêncio para um modo mais fraco entregaria exatamente a
/// garantia que se pediu.
///
/// `None` desliga o TLS. `Required` cifra sem conferir a ponta — é o que
/// significa pedir sigilo sem cadeia de confiança — e por isso aceita
/// certificado inválido de propósito.
fn ssl_opts(secrets: &InfraSecrets) -> anyhow::Result<Option<SslOpts>> {
    match secrets.ssl_mode {
        DatabaseSslMode::Disabled => Ok(None),
        DatabaseSslMode::Required => Ok(Some(
            SslOpts::default().with_danger_accept_invalid_certs(true),
        )),
        DatabaseSslMode::VerifyCa => {
            let Some(ca_path) = secrets.ssl_ca_path.as_deref() else {
                bail!("ssl_mode verify_ca exige o caminho do bundle da CA");
            };

            Ok(Some(
                SslOpts::default()
                    .with_root_certs(vec![PathBuf::from(ca_path).into()])
                    .with_danger_skip_domain_validation(!secrets.ssl_verify_hostname),
            ))
        }
    }
}
