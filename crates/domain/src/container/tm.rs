//! Regras de contêiner: o que é um contêiner válido e quando ele pode mudar de
//! status.
//!
//! Dois tipos de recusa moram aqui, e o `api-http` os traduz em respostas
//! diferentes:
//!
//! * **validação** — os dados enviados estão errados (422);
//! * **conflito** — os dados estão certos, mas o pátio não está num estado em
//!   que a operação faça sentido (409).
//!
//! A distinção é o que permite ao cliente saber se deve corrigir o formulário ou
//! olhar o contêiner.
//!
//! Nenhuma transição muta o contêiner recebido: cada uma devolve um objeto novo,
//! de modo que uma recusa deixa o chamador exatamente como estava.

use crate::enums::ContainerStatus;
use crate::error::{ContainerError, Validation};
use crate::id::IntIdGenerator;

use super::model::{Container, ContainerModel};

/// Constrói contêineres e é dono de todas as suas transições de status.
pub trait ContainerTM {
    /// Cria um contêiner novo, vazio e sem peso.
    fn create(&self, code: String, max_capacity: f64)
        -> Result<Box<dyn Container>, ContainerError>;

    /// Produz o contêiner com outra capacidade.
    fn update(
        &self,
        container: &dyn Container,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError>;

    /// Sela o contêiner, fechando-o para carga.
    fn seal(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;

    /// Despacha o contêiner selado.
    fn dispatch(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;
}

/// Comprimento máximo do código, casando com a coluna `VARCHAR(255)`.
const MAX_CODE_LENGTH: usize = 255;

/// Fração da capacidade que um contêiner precisa ter para ser selado.
///
/// Impede que um contêiner quase vazio saia do pátio como se fosse um
/// carregamento.
const MIN_SEAL_FILL_RATIO: f64 = 0.10;

/// A implementação, genérica sobre o gerador de id.
pub(crate) struct ContainerTMImpl<G> {
    id_generator: G,
}

impl<G: IntIdGenerator> ContainerTMImpl<G> {
    /// Monta o TableModule com o seu gerador de id.
    pub(crate) fn new(id_generator: G) -> Self {
        Self { id_generator }
    }

    /// Examina código e capacidade, acumulando o que estiver errado.
    fn validate(&self, code: &str, max_capacity: f64) -> Validation {
        let mut errors = Validation::new();

        if code.trim().is_empty() {
            errors.add("code", "Code is required.");
        } else if code.chars().count() > MAX_CODE_LENGTH {
            errors.add(
                "code",
                format!("Code must not exceed {MAX_CODE_LENGTH} characters."),
            );
        }

        errors.add_if(
            !max_capacity.is_finite() || max_capacity <= 0.0,
            "max_capacity",
            "Max capacity must be greater than zero.",
        );

        errors
    }

    /// Produz o contêiner com outro status.
    fn with_status(
        &self,
        container: &dyn Container,
        status: ContainerStatus,
    ) -> Box<dyn Container> {
        let mut model = ContainerModel::from_domain(container);
        model.set_weight_and_status(container.current_weight(), status);
        Box::new(model)
    }
}

impl<G: IntIdGenerator> ContainerTM for ContainerTMImpl<G> {
    fn create(
        &self,
        code: String,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError> {
        self.validate(&code, max_capacity)
            .into_result(())
            .map_err(ContainerError::Validation)?;

        Ok(Box::new(ContainerModel::new(
            self.id_generator.next(),
            code,
            0.0,
            max_capacity,
            ContainerStatus::Empty,
        )))
    }

    fn update(
        &self,
        container: &dyn Container,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError> {
        // O código existente é revalidado junto da capacidade nova. É barato, e
        // impede que uma linha anterior a uma regra mais rígida seja gravada de
        // volta sem passar por ela.
        self.validate(container.code(), max_capacity)
            .into_result(())
            .map_err(ContainerError::Validation)?;

        let mut model = ContainerModel::from_domain(container);
        model.set_max_capacity(max_capacity);
        Ok(Box::new(model))
    }

    fn seal(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError> {
        // As duas condições são independentes: `Loading` descarta o que está
        // vazio ou já selado, e a razão de enchimento descarta o que tem carga
        // de menos para valer a viagem.
        if container.status() != ContainerStatus::Loading {
            return Err(ContainerError::SealRequiresLoading);
        }

        if container.current_weight() < MIN_SEAL_FILL_RATIO * container.max_capacity() {
            return Err(ContainerError::SealBelowMinimumFill);
        }

        Ok(self.with_status(container, ContainerStatus::Sealed))
    }

    fn dispatch(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError> {
        // Exigir `Sealed` é também o que torna a operação idempotente no sentido
        // útil: o primeiro despacho deixa o contêiner `InTransit`, então o
        // segundo é recusado em vez de despachar duas vezes.
        if container.status() != ContainerStatus::Sealed {
            return Err(ContainerError::DispatchRequiresSealed);
        }

        Ok(self.with_status(container, ContainerStatus::InTransit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::tm::fields_of;
    use pretty_assertions::assert_eq;

    struct FixedIdGenerator;
    impl IntIdGenerator for FixedIdGenerator {
        fn next(&self) -> String {
            "C1".to_string()
        }
    }

    fn table_module() -> ContainerTMImpl<FixedIdGenerator> {
        ContainerTMImpl::new(FixedIdGenerator)
    }

    /// Contêiner com peso e status arbitrários, para exercitar as transições.
    fn container_at(weight: f64, capacity: f64, status: ContainerStatus) -> Box<dyn Container> {
        Box::new(ContainerModel::new(
            "C1".into(),
            "MSCU1234567".into(),
            weight,
            capacity,
            status,
        ))
    }

    #[test]
    fn nasce_vazio_e_sem_peso() {
        let container = table_module()
            .create("MSCU1234567".into(), 1000.0)
            .expect("os dados são válidos");

        assert_eq!(container.status(), ContainerStatus::Empty);
        assert_eq!(container.current_weight(), 0.0);
        assert_eq!(container.max_capacity(), 1000.0);
    }

    #[test]
    fn recusa_codigo_e_capacidade_invalidos() {
        let error = table_module()
            .create("".into(), 0.0)
            .err()
            .expect("os dois campos são inválidos");

        let ContainerError::Validation(fields) = error else {
            panic!("esperava erro de validação");
        };
        assert_eq!(fields_of(&fields), vec!["code", "max_capacity"]);
    }

    #[test]
    fn sela_um_conteiner_carregado_o_bastante() {
        let container = container_at(100.0, 1000.0, ContainerStatus::Loading);
        let sealed = table_module()
            .seal(container.as_ref())
            .expect("10% da capacidade é exatamente o mínimo");

        assert_eq!(sealed.status(), ContainerStatus::Sealed);
        // A transição não tocou o original.
        assert_eq!(container.status(), ContainerStatus::Loading);
    }

    #[test]
    fn recusa_selar_abaixo_do_minimo() {
        let container = container_at(99.9, 1000.0, ContainerStatus::Loading);
        let error = table_module()
            .seal(container.as_ref())
            .err()
            .expect("abaixo de 10% não sela");

        assert!(matches!(error, ContainerError::SealBelowMinimumFill));
    }

    #[test]
    fn so_sela_o_que_esta_carregando() {
        for status in [
            ContainerStatus::Empty,
            ContainerStatus::Sealed,
            ContainerStatus::InTransit,
        ] {
            let container = container_at(500.0, 1000.0, status);
            let error = table_module()
                .seal(container.as_ref())
                .err()
                .expect("{status} não pode ser selado");

            assert!(
                matches!(error, ContainerError::SealRequiresLoading),
                "status: {status}"
            );
        }
    }

    #[test]
    fn despacha_o_que_esta_selado() {
        let container = container_at(500.0, 1000.0, ContainerStatus::Sealed);
        let dispatched = table_module()
            .dispatch(container.as_ref())
            .expect("selado pode ser despachado");

        assert_eq!(dispatched.status(), ContainerStatus::InTransit);
    }

    #[test]
    fn o_segundo_despacho_e_recusado() {
        // É o que impede despachar duas vezes: depois do primeiro, o contêiner
        // não está mais `Sealed`.
        let dispatched = container_at(500.0, 1000.0, ContainerStatus::InTransit);
        let error = table_module()
            .dispatch(dispatched.as_ref())
            .err()
            .expect("já despachado não despacha de novo");

        assert!(matches!(error, ContainerError::DispatchRequiresSealed));
    }
}
