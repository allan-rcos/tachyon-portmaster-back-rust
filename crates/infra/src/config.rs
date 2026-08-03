//! Configuração da `infra`, nos dois eixos que ela tem.
//!
//! **Segredos e endpoints** chegam em runtime, na [`InfraSecrets`] que a
//! apresentação monta e passa ao `register`. **Knobs de arquitetura** — tamanho
//! do pool, capacidade e TTL de cache — são `const` decididas no build.
//!
//! A separação não é estética. O tamanho do pool não muda entre dois deploys do
//! mesmo binário e não deveria ser um `if` em produção; a senha do banco muda a
//! cada ambiente e não pode estar compilada dentro dele.

/// O texto secreto que a URI de conexão carrega.
///
/// Reexportado porque aparece num campo público de [`InfraSecrets`]: quem monta
/// os segredos precisa nomear o tipo, e obrigá-lo a declarar `secrecy` só para
/// isso acoplaria a apresentação a uma escolha desta camada.
pub use secrecy::SecretString;

/// Como a conexão com o banco trata TLS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatabaseSslMode {
    /// Sem TLS.
    ///
    /// A resposta certa para um banco em `127.0.0.1` ou numa subnet privada, e a
    /// errada para qualquer banco gerenciado — que recusa conexão em claro.
    #[default]
    Disabled,

    /// Criptografa sem validar o certificado.
    ///
    /// Resolve escuta passiva, não ataque ativo: quem consegue se pôr no meio
    /// apresenta o certificado que quiser.
    Required,

    /// Criptografa e valida a cadeia contra a CA configurada.
    VerifyCa,
}

/// Segredos e endpoints de runtime da `infra`.
#[derive(Debug, Clone)]
pub struct InfraSecrets {
    /// URI de conexão, com a senha.
    pub database_uri: SecretString,

    /// Como tratar TLS na conexão.
    pub ssl_mode: DatabaseSslMode,

    /// Bundle da CA, lido apenas em [`DatabaseSslMode::VerifyCa`].
    pub ssl_ca_path: Option<String>,

    /// Se o nome no certificado deve casar com o host.
    pub ssl_verify_hostname: bool,
}

/// Conexões máximas no pool.
///
/// Não é runtime: é decisão de arquitetura, e o valor certo depende de quantas
/// conexões o MariaDB do outro lado aguenta — que também não muda por deploy.
#[cfg(feature = "pool-default")]
pub(crate) const POOL_MAX_CONNECTIONS: u32 = 32;

/// Conexões máximas no pool, para instalações grandes.
#[cfg(all(feature = "pool-large", not(feature = "pool-default")))]
pub(crate) const POOL_MAX_CONNECTIONS: u32 = 128;

/// Quanto tempo uma consulta cacheada continua válida.
///
/// Curto de propósito. O cache aqui absorve rajadas de leitura repetida, não
/// substitui o banco — e toda escrita invalida a chave que tocou, então a
/// janela de dado velho é o intervalo entre duas leituras, não este TTL.
pub(crate) const READ_CACHE_TTL_SECONDS: u64 = 30;

/// Quantas consultas cacheadas cabem antes de o Moka começar a despejar.
pub(crate) const READ_CACHE_CAPACITY: u64 = 10_000;

/// Quantos marcadores cabem em memória.
///
/// Um marcador é uma sessão de refresh viva; o teto existe para que uma enxurrada
/// de logins não consuma memória sem limite.
pub(crate) const MARKER_CACHE_CAPACITY: u64 = 100_000;

/// Quantos metadados de sistema cabem em memória.
///
/// Permissões e grupos são dezenas, registrados no boot e nunca mais alterados.
pub(crate) const METADATA_CACHE_CAPACITY: u64 = 1_000;
