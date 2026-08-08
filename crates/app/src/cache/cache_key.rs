//! As chaves do cache de leitura.
//!
//! Namespace: o prefixo por recurso e o construtor da chave são a mesma
//! decisão. Separá-los deixaria a chance de alguém montar uma chave com um
//! prefixo que a invalidação não conhece — e a entrada nunca mais sairia.

/// Monta as chaves do cache de leitura.
pub(crate) struct CacheKey;

impl CacheKey {
    /// Leituras de conta — o próprio usuário e seus papéis.
    pub const ACCOUNT: &str = "account:";
    /// Leituras de contêiner, inclusive o resumo com carga e telemetria.
    pub const CONTAINER: &str = "container:";
    /// O painel do pátio.
    pub const METRICS: &str = "metrics:";
    /// Leituras de produto.
    pub const PRODUCT: &str = "product:";
    /// Leituras de papel.
    pub const ROLE: &str = "role:";
    /// Leituras de usuário.
    pub const USER: &str = "user:";

    /// Monta a chave de uma leitura a partir dos seus parâmetros.
    ///
    /// Todo parâmetro entra, mesmo ausente (como string vazia): uma chave que omite
    /// o filtro nulo faria "sem busca" e "busca vazia" colidirem, e a segunda
    /// receberia a resposta da primeira.
    pub(crate) fn of(prefix: &str, operation: &str, parts: &[&str]) -> String {
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "soma de dois len() para dimensionar a String: estourar usize exigiria uma chave maior que a memória"
        )]
        let mut key = String::with_capacity(prefix.len() + operation.len() + 16);

        key.push_str(prefix);
        key.push_str(operation);

        for part in parts {
            key.push(':');
            key.push_str(part);
        }

        key
    }
}
