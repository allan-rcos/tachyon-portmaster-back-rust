//! A implementação das regras de contêiner.

use nutype::nutype;

use crate::domain::Container;
use crate::enums::ContainerStatus;
use crate::error::{ContainerError, FieldError};
use crate::id::DatabaseIdGenerator;
use crate::table_modules::intern::models::container_model::ContainerModel;
use crate::table_modules::ContainerTM;

/// O código de um contêiner.
#[nutype(sanitize(trim), validate(not_empty, len_char_max = 255))]
struct ContainerCode(String);

/// A capacidade máxima de um contêiner, em toneladas.
///
/// O `finite` vem antes do `greater` porque `f64::NAN > 0.0` é **falso**: sem
/// ele, um NaN passaria e envenenaria toda comparação de peso adiante.
#[nutype(validate(finite, greater = 0.0))]
struct MaxCapacity(f64);

/// A implementação, genérica sobre o gerador de id.
#[derive(Clone)]
pub(crate) struct ContainerTMImpl<G> {
    /// De onde sai a identidade de um contêiner novo.
    id_generator: G,
}

impl<G: DatabaseIdGenerator> ContainerTMImpl<G> {
    /// Monta o `TableModule` com o seu gerador de id.
    pub(crate) const fn new(id_generator: G) -> Self {
        Self { id_generator }
    }

    /// Examina código e capacidade, acumulando o que estiver errado.
    fn checked(
        code: String,
        max_capacity: f64,
    ) -> Result<(ContainerCode, MaxCapacity), ContainerError> {
        let checked_code = ContainerCode::try_new(code);
        let checked_capacity = MaxCapacity::try_new(max_capacity);

        let mut errors = Vec::new();
        if let Err(error) = &checked_code {
            errors.push(code_refused(error));
        }
        if checked_capacity.is_err() {
            errors.push(FieldError::new(
                "max_capacity",
                "Max capacity must be greater than zero.",
            ));
        }

        let (Ok(code), Ok(capacity)) = (checked_code, checked_capacity) else {
            return Err(ContainerError::Validation(errors));
        };

        Ok((code, capacity))
    }

    /// Produz o contêiner com outro status.
    fn with_status(container: &dyn Container, status: ContainerStatus) -> Box<dyn Container> {
        let mut model = ContainerModel::from_domain(container);
        model.set_weight_and_status(container.current_weight(), status);
        Box::new(model)
    }
}

impl<G: DatabaseIdGenerator> ContainerTM for ContainerTMImpl<G> {
    fn create(
        &self,
        code: String,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError> {
        let (code, max_capacity) = Self::checked(code, max_capacity)?;

        Ok(Box::new(ContainerModel::new(
            self.id_generator.next(),
            code.into_inner(),
            0.0,
            max_capacity.into_inner(),
            ContainerStatus::Empty,
        )))
    }

    /// Produz o contêiner com outra capacidade.
    ///
    /// O código **existente** é revalidado junto da capacidade nova. É barato,
    /// e impede que uma linha anterior a uma regra mais rígida seja gravada de
    /// volta sem passar por ela.
    fn update(
        &self,
        container: &dyn Container,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError> {
        let (_, max_capacity) = Self::checked(container.code().to_owned(), max_capacity)?;

        let mut model = ContainerModel::from_domain(container);
        model.set_max_capacity(max_capacity.into_inner());
        Ok(Box::new(model))
    }

    /// Sela o contêiner, se ele estiver carregando e suficientemente cheio.
    ///
    /// As duas condições são **independentes**: `Loading` descarta o que está
    /// vazio ou já selado, e a razão de enchimento descarta o que tem carga de
    /// menos para valer a viagem.
    fn seal(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError> {
        if container.status() != ContainerStatus::Loading {
            return Err(ContainerError::SealRequiresLoading);
        }

        if container.current_weight() < MIN_SEAL_FILL_RATIO * container.max_capacity() {
            return Err(ContainerError::SealBelowMinimumFill);
        }

        Ok(Self::with_status(container, ContainerStatus::Sealed))
    }

    /// Despacha um contêiner selado.
    ///
    /// Exigir `Sealed` é também o que torna a operação idempotente no sentido
    /// útil: o primeiro despacho deixa o contêiner `InTransit`, então o segundo
    /// é recusado em vez de despachar duas vezes.
    fn dispatch(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError> {
        if container.status() != ContainerStatus::Sealed {
            return Err(ContainerError::DispatchRequiresSealed);
        }

        Ok(Self::with_status(container, ContainerStatus::InTransit))
    }
}

/// Comprimento máximo do código, casando com a coluna `VARCHAR(255)`.
const MAX_CODE_LENGTH: usize = 255;

/// Traduz a recusa do código na mensagem que o cliente lê.
fn code_refused(error: &ContainerCodeError) -> FieldError {
    match *error {
        ContainerCodeError::NotEmptyViolated => FieldError::new("code", "Code is required."),
        ContainerCodeError::LenCharMaxViolated => FieldError::new(
            "code",
            format!("Code must not exceed {MAX_CODE_LENGTH} characters."),
        ),
    }
}

/// Fração da capacidade que um contêiner precisa ter para ser selado.
///
/// Impede que um contêiner quase vazio saia do pátio como se fosse um
/// carregamento.
const MIN_SEAL_FILL_RATIO: f64 = 0.10;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_modules::intern::helpers::fields_of::fields_of;
    use pretty_assertions::assert_eq;

    struct FixedIdGenerator;
    impl DatabaseIdGenerator for FixedIdGenerator {
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
            .create(String::new(), 0.0)
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
                .unwrap_or_else(|| panic!("{status} não pode ser selado"));

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

    /// Depois do primeiro despacho o contêiner não está mais `Sealed`.
    ///
    /// É o que impede despachar duas vezes.
    #[test]
    fn o_segundo_despacho_e_recusado() {
        let dispatched = container_at(500.0, 1000.0, ContainerStatus::InTransit);
        let error = table_module()
            .dispatch(dispatched.as_ref())
            .err()
            .expect("já despachado não despacha de novo");

        assert!(matches!(error, ContainerError::DispatchRequiresSealed));
    }
}
