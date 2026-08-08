//! A leitura de um contêiner e dos seus satélites a partir de uma linha.

use crate::entity::codec::Codec;
use crate::query::row::Row;
use crate::query::views::{CargoItemView, ContainerViewItem, TelemetryLogView};
use anyhow::Context;
use portmaster_domain::enums::{ContainerStatus, TelemetryEvent};
use serde_json::Value;
use sqlx::mysql::MySqlRow;

/// Lê contêiner, carga e telemetria de uma linha de consulta.
pub(crate) struct ContainerReader;

impl ContainerReader {
    /// Uma linha de `containers` como a View a quer.
    pub(crate) fn item(row: &MySqlRow) -> anyhow::Result<ContainerViewItem> {
        Ok(ContainerViewItem {
            id: Row::id(row, "id")?,
            code: Row::text(row, "code")?,
            current_weight: Row::real(row, "current_weight")?,
            max_capacity: Row::real(row, "max_capacity")?,
            status: Row::enum_index(row, "status", ContainerStatus::from_i32, "ContainerStatus")?,
        })
    }

    /// O manifesto agregado pelo banco.
    ///
    /// `JSON_ARRAYAGG` devolve `NULL` quando não há linha nenhuma — contêiner vazio,
    /// que é estado normal e não ausência de dado.
    pub(crate) fn manifest_of(json: Option<&str>) -> anyhow::Result<Vec<CargoItemView>> {
        let entries = Self::entries_of(json, "manifest_json")?;
        let mut items = Vec::with_capacity(entries.len());

        for entry in &entries {
            items.push(CargoItemView {
                product_id: Codec::encode_id(Self::int_of(entry, "product_id")?),
                product_name: Self::str_of(entry, "product_name")?,
                quantity: Self::float_of(entry, "quantity")?,
                weight: Self::float_of(entry, "weight")?,
            });
        }

        Ok(items)
    }

    /// A telemetria recente agregada pelo banco.
    pub(crate) fn logs_of(json: Option<&str>) -> anyhow::Result<Vec<TelemetryLogView>> {
        let entries = Self::entries_of(json, "logs_json")?;
        let mut logs = Vec::with_capacity(entries.len());

        for entry in &entries {
            let event = i32::try_from(Self::int_of(entry, "event")?).unwrap_or(-1);

            // Um evento gravado que não corresponde a variante nenhuma é descartado,
            // não aproximado. O campo do fio é um enum: não existe valor que
            // signifique "aconteceu algo, mas nenhum destes", então escolher uma
            // variante reportaria um evento que nunca ocorreu. A linha continua em
            // `telemetry_logs` de qualquer forma.
            if TelemetryEvent::from_i32(event).is_none() {
                continue;
            }

            logs.push(TelemetryLogView {
                id: Codec::encode_id(Self::int_of(entry, "id")?),
                event,
                description: entry
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                timestamp: Self::int_of(entry, "timestamp")?,
            });
        }

        Ok(logs)
    }

    /// As entradas de um array JSON agregado pelo banco.
    pub(crate) fn entries_of(json: Option<&str>, column: &str) -> anyhow::Result<Vec<Value>> {
        let Some(json) = json else {
            return Ok(Vec::new());
        };

        let parsed: Value = serde_json::from_str(json)
            .with_context(|| format!("coluna `{column}` não é JSON válido"))?;

        Ok(parsed.as_array().cloned().unwrap_or_default())
    }

    /// Um inteiro de dentro do JSON agregado.
    pub(crate) fn int_of(entry: &Value, field: &str) -> anyhow::Result<i64> {
        entry
            .get(field)
            .and_then(Value::as_i64)
            .with_context(|| format!("campo `{field}` do JSON agregado não é inteiro"))
    }

    /// Um real de dentro do JSON agregado.
    pub(crate) fn float_of(entry: &Value, field: &str) -> anyhow::Result<f64> {
        entry
            .get(field)
            .and_then(Value::as_f64)
            .with_context(|| format!("campo `{field}` do JSON agregado não é real"))
    }

    /// Um texto de dentro do JSON agregado.
    pub(crate) fn str_of(entry: &Value, field: &str) -> anyhow::Result<String> {
        entry
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .with_context(|| format!("campo `{field}` do JSON agregado não é texto"))
    }
}
