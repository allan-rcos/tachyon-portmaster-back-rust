//! A falha que um caso de uso devolve.

use portmaster_domain::error::{
    AuthError, ContainerError, FieldError, ManifestError, MarkerError, MetadataError, ProductError,
    RoleError, UserError,
};

/// O que pode dar errado num caso de uso.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Campos inválidos, em lote.
    ///
    /// Lista e não primeiro-erro: quem preencheu um formulário quer corrigir
    /// tudo de uma vez, não descobrir um problema por requisição.
    #[error("dados inválidos")]
    Validation(Vec<FieldError>),

    /// Credenciais recusadas, ou sessão que não vale mais.
    ///
    /// E-mail desconhecido e senha errada produzem **o mesmo** erro de
    /// propósito: distinguir os dois entregaria a quem tentasse adivinhar uma
    /// lista de e-mails cadastrados.
    #[error("Invalid e-mail or password.")]
    Unauthenticated,

    /// Falta a permissão exigida.
    ///
    /// O slug viaja para o log. O `api-http` **não** o coloca no corpo — é o
    /// comportamento que o PHP tinha, e dizer ao cliente qual permissão faltou
    /// descreve para ele o mapa de autorização do sistema.
    #[error("permissão negada: {permission}")]
    Forbidden {
        /// O slug que faltava.
        permission: String,
    },

    /// O recurso pedido não existe.
    #[error("{resource} não encontrado: {id}")]
    NotFound {
        /// Que tipo de recurso.
        resource: &'static str,
        /// O id procurado.
        id: String,
    },

    /// A operação contradiz o estado atual.
    #[error("{0}")]
    Conflict(String),

    /// Regra de autenticação do domínio.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// Regra de contêiner que o **estado** viola — selar o que não está
    /// carregando, despachar o que não está selado.
    ///
    /// Sem `#[from]`: `ContainerError` também carrega campos recusados, e
    /// aqueles são validação, não conflito. A conversão abaixo separa os dois.
    #[error(transparent)]
    Container(ContainerError),

    /// Regra de manifesto que o **estado** viola — contêiner fechado, carga
    /// além da capacidade, desembarque maior do que o embarcado.
    ///
    /// Sem `#[from]` pela mesma razão de [`Self::Container`].
    #[error(transparent)]
    Manifest(ManifestError),

    /// Regra de marcador.
    #[error(transparent)]
    Marker(#[from] MarkerError),

    /// Regra de metadado de sistema.
    #[error(transparent)]
    Metadata(#[from] MetadataError),

    /// Falha de I/O ou de infraestrutura.
    ///
    /// Opaca de propósito: "o banco recusou a conexão" não é uma regra que o
    /// cliente possa satisfazer, e detalhar o motivo só vazaria topologia.
    #[error(transparent)]
    Infra(#[from] anyhow::Error),
}

impl AppError {
    /// O erro de um recurso que deveria existir e não existe.
    pub(crate) fn not_found(resource: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            id: id.into(),
        }
    }
}

impl From<UserError> for AppError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Validation(fields) => Self::Validation(fields),
        }
    }
}

impl From<RoleError> for AppError {
    fn from(error: RoleError) -> Self {
        match error {
            RoleError::Validation(fields) => Self::Validation(fields),
        }
    }
}

impl From<ProductError> for AppError {
    fn from(error: ProductError) -> Self {
        match error {
            ProductError::Validation(fields) => Self::Validation(fields),
        }
    }
}

impl From<ContainerError> for AppError {
    fn from(error: ContainerError) -> Self {
        match error {
            ContainerError::Validation(fields) => Self::Validation(fields),
            rule => Self::Container(rule),
        }
    }
}

impl From<ManifestError> for AppError {
    /// Traduz o erro de manifesto, separando validação de conflito.
    ///
    /// "Quantidade tem que ser maior que zero" descreve o **campo** que chegou,
    /// não o estado do contêiner: é o mesmo tipo de recusa que um nome vazio, e
    /// sai como 422 junto com os outros. As demais variantes falam do pátio, e
    /// continuam sendo conflito.
    fn from(error: ManifestError) -> Self {
        match error {
            ManifestError::InvalidQuantity => Self::Validation(vec![FieldError::new(
                "quantity",
                ManifestError::InvalidQuantity.to_string(),
            )]),
            rule => Self::Manifest(rule),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// A mensagem é a mesma para e-mail desconhecido e senha errada.
    ///
    /// Se divergissem, bastaria comparar as duas para enumerar quem tem conta.
    #[test]
    fn credencial_ruim_nao_diz_qual_metade_falhou() {
        assert_eq!(
            AppError::Unauthenticated.to_string(),
            "Invalid e-mail or password."
        );
    }

    #[test]
    fn a_validacao_do_dominio_chega_como_lote() {
        let error: AppError = UserError::Validation(vec![
            FieldError::new("email", "malformado"),
            FieldError::new("password", "curta demais"),
        ])
        .into();

        let AppError::Validation(fields) = error else {
            panic!("erro de validação do domínio deveria virar Validation");
        };
        assert_eq!(fields.len(), 2, "o lote não pode perder campos no caminho");
    }

    /// "maior que zero" descreve o corpo que chegou, e o cliente corrige o
    /// corpo.
    ///
    /// As outras variantes descrevem o pátio, e ele não tem como corrigir
    /// aquilo reenviando a mesma coisa.
    #[test]
    fn quantidade_invalida_e_validacao_e_nao_conflito() {
        let error: AppError = ManifestError::InvalidQuantity.into();

        let AppError::Validation(fields) = error else {
            panic!("quantidade inválida deveria virar Validation");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "quantity");
    }

    #[test]
    fn as_regras_de_estado_continuam_conflito() {
        assert!(matches!(
            AppError::from(ManifestError::ExceedsCapacity),
            AppError::Manifest(_)
        ));
        assert!(matches!(
            AppError::from(ContainerError::SealRequiresLoading),
            AppError::Container(_)
        ));
    }

    /// `ContainerError` carrega os dois tipos de recusa; sem separá-las, um
    /// código de contêiner malformado sairia como 409.
    #[test]
    fn campo_recusado_de_conteiner_nao_vira_conflito() {
        let error: AppError =
            ContainerError::Validation(vec![FieldError::new("code", "vazio")]).into();

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn o_slug_negado_fica_no_erro() {
        // Vai para o log; quem decide não mostrá-lo ao cliente é o api-http.
        let error = AppError::Forbidden {
            permission: "product:create".into(),
        };

        assert!(error.to_string().contains("product:create"));
    }
}
