//! Quem está agindo.
//!
//! O contexto chega ao UseCase **por argumento**, como propriedade do Command —
//! nunca de um estado global. A diferença não é estética: um UseCase que vai
//! buscar contexto num task-local só funciona dentro de uma requisição HTTP, e
//! passa a mentir sobre as próprias dependências. Recebendo por argumento, ele é
//! testável com um contexto montado à mão e a assinatura diz a verdade.
//!
//! ## Por que é um DTO plano, e não `Box<dyn User>`
//!
//! Este contexto nasce do JWT, e a regra de ouro da autenticação é que **nenhum
//! middleware toca o banco**. Um `Box<dyn User>` exigiria um model do `domain`,
//! que só um TableModule constrói, que por sua vez exigiria carregar o usuário —
//! uma consulta por requisição, exatamente o que a sessão stateless evita.
//!
//! O que a autorização precisa saber cabe no token: quem é, e o que os papéis
//! dele concedem.

/// O usuário da requisição, como o token o descreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserContext {
    /// Id em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
    /// Os papéis que ele carrega.
    pub roles: Vec<RoleContext>,
}

impl UserContext {
    /// Se algum papel concede a permissão.
    ///
    /// Basta **um**: papéis somam permissões, não as restringem.
    pub fn has_permission(&self, slug: &str) -> bool {
        self.roles.iter().any(|role| role.grants(slug))
    }
}

/// Um papel e o que ele concede.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleContext {
    /// Id em base62.
    pub id: String,
    /// Nome do papel.
    pub name: String,
    /// Os slugs de permissão concedidos.
    pub permissions: Vec<String>,
}

impl RoleContext {
    /// Se este papel concede a permissão.
    pub fn grants(&self, slug: &str) -> bool {
        self.permissions.iter().any(|granted| granted == slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(name: &str, permissions: &[&str]) -> RoleContext {
        RoleContext {
            id: "1".into(),
            name: name.into(),
            permissions: permissions.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    fn user(roles: Vec<RoleContext>) -> UserContext {
        UserContext {
            id: "1".into(),
            name: "Ana".into(),
            email: "ana@portmaster.local".into(),
            roles,
        }
    }

    #[test]
    fn qualquer_papel_basta_para_conceder() {
        // Papéis somam. Exigir que todos concedam faria acrescentar um papel
        // **reduzir** o que o usuário pode fazer.
        let context = user(vec![
            role("Leitor", &["product:read"]),
            role("Operador", &["container:seal"]),
        ]);

        assert!(context.has_permission("product:read"));
        assert!(context.has_permission("container:seal"));
    }

    #[test]
    fn sem_papel_nenhum_nao_ha_permissao() {
        assert!(!user(Vec::new()).has_permission("product:read"));
    }

    #[test]
    fn a_comparacao_de_slug_e_exata() {
        // `product:read` não pode ser satisfeita por `product:read-all` nem por
        // um prefixo — permissão não tem hierarquia neste sistema.
        let context = user(vec![role("Quase", &["product:read-all"])]);

        assert!(!context.has_permission("product:read"));
    }
}
