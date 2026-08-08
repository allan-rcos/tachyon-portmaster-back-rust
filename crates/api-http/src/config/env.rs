//! Os nomes das variáveis de ambiente.

/// Os nomes das variáveis, exatamente como o PHP as lia.
///
/// São contrato com o `docker-compose.yml` e com quem já roda o sistema:
/// renomear qualquer uma quebra um deploy existente em silêncio, porque o valor
/// simplesmente cai no padrão.
///
/// Struct-namespace e não um `mod` de consts soltas: o módulo já é o arquivo, e
/// agrupá-las num tipo é o que mantém um export só por arquivo.
pub(crate) struct Env;

impl Env {
    /// `APP_HOST`
    pub(crate) const HOST: &str = "APP_HOST";
    /// `APP_PORT`
    pub(crate) const PORT: &str = "APP_PORT";
    /// `APP_ENV`
    pub(crate) const ENVIRONMENT: &str = "APP_ENV";
    /// `APP_REQUEST_TIMEOUT`
    pub(crate) const REQUEST_TIMEOUT: &str = "APP_REQUEST_TIMEOUT";
    /// `APP_DB_HOST`
    pub(crate) const DB_HOST: &str = "APP_DB_HOST";
    /// `APP_DB_PORT`
    pub(crate) const DB_PORT: &str = "APP_DB_PORT";
    /// `APP_DB_NAME`
    pub(crate) const DB_NAME: &str = "APP_DB_NAME";
    /// `APP_DB_USER`
    pub(crate) const DB_USER: &str = "APP_DB_USER";
    /// `APP_DB_PASSWORD`
    pub(crate) const DB_PASSWORD: &str = "APP_DB_PASSWORD";
    /// `APP_DB_SSL_MODE`
    pub(crate) const DB_SSL_MODE: &str = "APP_DB_SSL_MODE";
    /// `APP_DB_SSL_CA`
    pub(crate) const DB_SSL_CA: &str = "APP_DB_SSL_CA";
    /// `APP_DB_SSL_VERIFY_CN`
    pub(crate) const DB_SSL_VERIFY_CN: &str = "APP_DB_SSL_VERIFY_CN";
    /// `APP_JWT_SECRET`
    pub(crate) const JWT_SECRET: &str = "APP_JWT_SECRET";
    /// `APP_JWT_TTL`
    pub(crate) const JWT_TTL: &str = "APP_JWT_TTL";
    /// `APP_JWT_ISSUER`
    pub(crate) const JWT_ISSUER: &str = "APP_JWT_ISSUER";
    /// `APP_JWT_COOKIE_NAME`
    pub(crate) const JWT_COOKIE_NAME: &str = "APP_JWT_COOKIE_NAME";
    /// `APP_JWT_COOKIE_SECURE`
    pub(crate) const JWT_COOKIE_SECURE: &str = "APP_JWT_COOKIE_SECURE";
    /// `APP_JWT_COOKIE_SAME_SITE`
    pub(crate) const JWT_COOKIE_SAME_SITE: &str = "APP_JWT_COOKIE_SAME_SITE";
    /// `APP_REFRESH_COOKIE_NAME`
    pub(crate) const REFRESH_COOKIE_NAME: &str = "APP_REFRESH_COOKIE_NAME";
    /// `APP_REFRESH_TTL`
    pub(crate) const REFRESH_TTL: &str = "APP_REFRESH_TTL";
    /// `APP_CLUSTER_ID`
    pub(crate) const CLUSTER_ID: &str = "APP_CLUSTER_ID";
    /// `APP_SERVER_ID`
    pub(crate) const SERVER_ID: &str = "APP_SERVER_ID";
    /// `APP_CORS_ORIGINS`
    pub(crate) const CORS_ORIGINS: &str = "APP_CORS_ORIGINS";
}
