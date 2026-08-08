//! O contrato de leitura de `Role`.

use chrono::{DateTime, Utc};

/// Um pacote de permissões que se concede a um usuário.
///
/// O trait é somente-leitura, como todo objeto de domínio.
pub trait Role: Send + Sync {
    /// Id em base62.
    fn id(&self) -> &str;

    /// Nome de exibição.
    fn name(&self) -> &str;

    /// Slugs de permissão que este papel concede, em `domain:action`.
    ///
    /// Slugs e não objetos `Permission`: o papel é persistido como JSON e
    /// sobrevive a qualquer registro em memória, cujos ids numéricos são
    /// reatribuídos a cada boot. O slug é a única referência estável.
    fn permissions(&self) -> &[String];

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;

    /// Quando mudou pela última vez.
    fn updated_at(&self) -> DateTime<Utc>;

    /// Quando foi removido, ou `None` enquanto vivo.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Duplica o papel atrás do trait.
    ///
    /// Existe porque um usuário carrega os seus papéis como `Box<dyn Role>` e
    /// precisa ser reconstruível a partir do trait: sem isto, recriar um usuário
    /// exigiria conhecer o tipo concreto por trás de cada papel, que é
    /// exatamente o que o trait esconde.
    fn clone_role(&self) -> Box<dyn Role>;
}
