//! As conversões de coluna que o derive não faz sozinho.
//!
//! O `FromRow` derivado converte uma coluna quando o tipo do campo sabe nascer
//! de um `Value`. Quatro dos tipos que as entities guardam não sabem — o
//! [`EntityId`], os dois enums de domínio e o `DateTime<Utc>` — e para esses o
//! derive aceita um par de funções por campo: uma que lê e uma que devolve o
//! valor à linha.
//!
//! Todas as quatro conversões partem de um inteiro, porque é isso que as colunas
//! guardam: id e epoch em `BIGINT`, índice de enum em `TINYINT`.
//!
//! ## Por que a devolução existe
//!
//! Quando um campo falha, o derive precisa recompor a linha inteira para
//! entregá-la no erro — inclusive os campos que já tinham sido lidos. Sem a
//! função de volta ele tentaria um `Into<Value>` que estes tipos não têm, e o
//! par ficaria pela metade.
//!
//! Só quem **precede** outro campo é recomposto, então `deleted_at`, último em
//! todas as entities, lê sem devolver. Um campo novo depois dele passa a exigir
//! a volta, e a falta dela é erro de compilação — não silêncio.

use chrono::{DateTime, Utc};
use mysql_async::{from_value_opt, FromValueError, Value};
use portmaster_domain::enums::{ContainerStatus, RiskClass};

use crate::entity::entity_id::EntityId;

/// Traduz entre a coluna e o tipo que a entity guarda.
///
/// Namespace no molde do [`Codec`](super::codec::Codec): cada par de funções é
/// a mesma fronteira vista dos dois lados, e nenhuma delas faz sentido fora de
/// um atributo de campo.
pub(crate) struct Decode;

impl Decode {
    /// Adota o id que veio da coluna.
    ///
    /// A recusa devolve o `Value` original e não a mensagem do [`EntityId`]: o
    /// derive só carrega o valor, e é ele que aparece na linha do
    /// `FromRowError`. Quem a lê ainda sabe qual consulta falhou, porque o
    /// repositório embrulha o erro com o id que pediu.
    pub fn entity_id(value: Value) -> Result<EntityId, FromValueError> {
        let raw: i64 = from_value_opt(value.clone())?;

        EntityId::try_from(raw).map_err(|_| FromValueError(value))
    }

    /// Devolve o id à linha, como a coluna o guarda.
    pub fn entity_id_value(id: EntityId) -> Value {
        Value::Int(id.into_raw())
    }

    /// Lê a coluna de tempo como o instante que o epoch nela nomeia.
    ///
    /// A coluna é `BIGINT` com epoch em milissegundos — um número com um
    /// significado só, que nenhuma variável de sessão reinterpreta. Um valor que
    /// não corresponde a instante nenhum é recusado em vez de saturado: data
    /// aproximada é data errada dita com confiança.
    pub fn utc(value: Value) -> Result<DateTime<Utc>, FromValueError> {
        let millis: i64 = from_value_opt(value.clone())?;

        DateTime::from_timestamp_millis(millis).ok_or(FromValueError(value))
    }

    /// Devolve o instante à linha, como o epoch que a coluna guarda.
    pub const fn utc_value(at: DateTime<Utc>) -> Value {
        Value::Int(at.timestamp_millis())
    }

    /// O mesmo que [`utc`](Self::utc), para a coluna que aceita nulo.
    ///
    /// `NULL` é ausência legítima aqui — é o soft-delete não aplicado — e não
    /// falha de conversão.
    pub fn utc_opt(value: Value) -> Result<Option<DateTime<Utc>>, FromValueError> {
        if value == Value::NULL {
            return Ok(None);
        }

        Self::utc(value).map(Some)
    }

    /// Lê o índice gravado como a classe de risco que ele nomeia.
    ///
    /// Um índice fora da faixa é recusado em vez de aproximado: escolher a
    /// variante mais próxima afirmaria uma classe de risco que o banco não
    /// guardou, e classe de risco errada é carga perigosa reportada como
    /// inofensiva.
    pub fn risk_class(value: Value) -> Result<RiskClass, FromValueError> {
        let index = Self::index(&value)?;

        RiskClass::try_from(index).map_err(|_| FromValueError(value))
    }

    /// Devolve a classe de risco à linha, como índice.
    pub fn risk_class_value(class: RiskClass) -> Value {
        Value::Int(i64::from(class.as_i32()))
    }

    /// Lê o índice gravado como o status que ele nomeia.
    ///
    /// Recusa pelo mesmo motivo da classe de risco: um status aproximado
    /// afirmaria um estado que a linha não guarda.
    pub fn container_status(value: Value) -> Result<ContainerStatus, FromValueError> {
        let index = Self::index(&value)?;

        ContainerStatus::try_from(index).map_err(|_| FromValueError(value))
    }

    /// Devolve o status à linha, como índice.
    pub fn container_status_value(status: ContainerStatus) -> Value {
        Value::Int(i64::from(status.as_i32()))
    }

    /// O índice de enum que a coluna guarda.
    ///
    /// A coluna é `TINYINT` e o driver a entrega como `i64`; os enums de
    /// domínio falam `i32`. O estreitamento é falível de propósito — um valor
    /// que não cabe em `i32` não corresponde a variante nenhuma de qualquer
    /// forma.
    fn index(value: &Value) -> Result<i32, FromValueError> {
        let raw: i64 = from_value_opt(value.clone())?;

        i32::try_from(raw).map_err(|_| FromValueError(value.clone()))
    }
}
