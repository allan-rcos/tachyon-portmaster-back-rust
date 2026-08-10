//! O contrato de leitura de `User`.

use chrono::{DateTime, Utc};

use crate::domain::Role;

/// Alguém que faz login, com os papéis que carrega.
///
/// Permissão nunca é concedida direto a uma pessoa — ela vem pelo papel, e é a
/// lista de slugs do papel que o guarda de autorização confere.
///
/// O trait é **somente-leitura**: só existem getters. Quem o recebe consegue ler
/// o usuário mas não alterá-lo, e é isso que impede o `app` ou o `api` de
/// mudarem um e-mail sem passar pela validação do `TableModule`.
pub trait User: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Nome de exibição.
    fn name(&self) -> &str;

    /// E-mail, que é também o identificador de login.
    fn email(&self) -> &str;

    /// Hash Argon2id da senha — nunca a senha.
    ///
    /// A senha em claro atravessa só o `TableModule`, o tempo de ser validada e
    /// hasheada; nada mais no sistema a retém.
    fn password_hash(&self) -> &str;

    /// Papéis atribuídos, na ordem em que foram concedidos.
    fn roles(&self) -> &[Box<dyn Role>];

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    ///
    /// Usuário é entidade forte: remover grava a data em vez de apagar a linha,
    /// e toda leitura filtra por `None`.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
}
