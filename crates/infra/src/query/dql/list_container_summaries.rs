//! A listagem de contêineres com carga e telemetria recente.
//!
//! ## As coleções aninhadas viram JSON no próprio banco
//!
//! A alternativa — uma consulta por contêiner para a carga e outra para os logs
//! — custaria 2n idas ao banco por página.
//!
//! O timestamp atravessa sem conversão nenhuma: a coluna guarda epoch em ms, o
//! `JSON_OBJECT` copia o número, e é assim que a View o quer. Como data, ele
//! sairia do JSON como texto, e a hidratação teria que interpretar formato de
//! data para chegar no mesmo número.
//!
//! ## Por que a janela de recentes é sub-consulta escalar
//!
//! O `MariaDB` não tem `LATERAL`, então uma tabela derivada não pode enxergar o
//! `c.id` de fora. Limitar pelo id do n-ésimo mais novo traz as mesmas linhas
//! com uma correlação que o motor aceita.

use anyhow::Context as _;
use mysql_async::{Params, Row, Value as SqlValue};
use portmaster_domain::enums::TelemetryEvent;
use serde_json::Value;

use crate::entity::codec::Codec;
use crate::query::column::Column;
use crate::query::cursor::{Cursor, CursorFilters};
use crate::query::dql::list_containers::read_item;
use crate::query::dql::paging::Paging;
use crate::query::views::{
    CargoItemView, ContainerSummaryListView, ContainerSummaryViewItem, TelemetryLogView,
};
use crate::query::{Dql, SqlDql};

/// Quantos registros de telemetria a janela recente traz.
const RECENT_LOGS: u32 = 10;

/// A listagem de contêineres com carga e telemetria recente.
///
/// `id` restringe a um contêiner só, quando a rota pediu um. Falível pelo mesmo
/// motivo das consultas por id: um base62 inventado não deve abrir transação.
pub fn list_container_summaries(
    cursor: Option<String>,
    limit: Option<u32>,
    id: Option<String>,
) -> anyhow::Result<impl SqlDql<View = ContainerSummaryListView>> {
    let raw_id = id.as_deref().map(Codec::decode_id).transpose()?;

    Ok(ListContainerSummaries {
        limit: Paging::effective_limit(limit),
        id,
        raw_id,
        cursor,
    })
}

/// A listagem de contêineres com os satélites deles.
struct ListContainerSummaries {
    /// O tamanho da página, já resolvido.
    limit: u32,
    /// O id pedido como o cliente o mandou — entra na identidade do cursor.
    id: Option<String>,
    /// O mesmo id, decodificado.
    raw_id: Option<i64>,
    /// O cursor como o cliente o mandou.
    cursor: Option<String>,
}

impl ListContainerSummaries {
    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        CursorFilters::of([
            ("limit", self.limit.to_string()),
            ("id", self.id.clone().unwrap_or_default()),
        ])
    }

    /// A restrição a um contêiner só, aplicada à página e à contagem.
    ///
    /// O mesmo nome nas duas ocorrências: o driver o repete onde ele aparecer, e
    /// ligar uma vez basta.
    fn id_condition(&self, alias: &str) -> String {
        self.raw_id
            .map_or_else(String::new, |_| format!(" AND {alias}id = :id"))
    }
}

/// O manifesto agregado pelo banco.
///
/// `JSON_ARRAYAGG` devolve `NULL` quando não há linha nenhuma — contêiner vazio,
/// que é estado normal e não ausência de dado.
fn read_manifest(json: Option<&str>) -> anyhow::Result<Vec<CargoItemView>> {
    let entries = read_entries(json, "manifest_json")?;
    let mut items = Vec::with_capacity(entries.len());

    for entry in &entries {
        items.push(CargoItemView {
            product_id: Codec::encode_id(int_of(entry, "product_id")?),
            product_name: str_of(entry, "product_name")?,
            quantity: float_of(entry, "quantity")?,
            weight: float_of(entry, "weight")?,
        });
    }

    Ok(items)
}

/// A telemetria recente, descartando evento que não reconhece.
///
/// Um evento gravado que não corresponde a variante nenhuma é **descartado, não
/// aproximado**. O campo do fio é um enum: não existe valor que signifique
/// "aconteceu algo, mas nenhum destes", então escolher uma variante reportaria
/// um evento que nunca ocorreu. A linha continua em `telemetry_logs` de
/// qualquer forma.
fn read_logs(json: Option<&str>) -> anyhow::Result<Vec<TelemetryLogView>> {
    let entries = read_entries(json, "logs_json")?;
    let mut logs = Vec::with_capacity(entries.len());

    for entry in &entries {
        let event = i32::try_from(int_of(entry, "event")?).unwrap_or(-1);

        if TelemetryEvent::from_i32(event).is_none() {
            continue;
        }

        logs.push(TelemetryLogView {
            id: Codec::encode_id(int_of(entry, "id")?),
            event,
            description: entry
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_owned),
            timestamp: int_of(entry, "timestamp")?,
        });
    }

    Ok(logs)
}

/// As entradas de um array JSON agregado pelo banco.
fn read_entries(json: Option<&str>, column: &str) -> anyhow::Result<Vec<Value>> {
    let Some(json) = json else {
        return Ok(Vec::new());
    };

    let parsed: Value = serde_json::from_str(json)
        .with_context(|| format!("coluna `{column}` não é JSON válido"))?;

    Ok(parsed.as_array().cloned().unwrap_or_default())
}

/// Um inteiro de dentro do JSON agregado.
fn int_of(entry: &Value, field: &str) -> anyhow::Result<i64> {
    entry
        .get(field)
        .and_then(Value::as_i64)
        .with_context(|| format!("campo `{field}` do JSON agregado não é inteiro"))
}

/// Um real de dentro do JSON agregado.
fn float_of(entry: &Value, field: &str) -> anyhow::Result<f64> {
    entry
        .get(field)
        .and_then(Value::as_f64)
        .with_context(|| format!("campo `{field}` do JSON agregado não é real"))
}

/// Um texto de dentro do JSON agregado.
fn str_of(entry: &Value, field: &str) -> anyhow::Result<String> {
    entry
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .with_context(|| format!("campo `{field}` do JSON agregado não é texto"))
}

impl Dql for ListContainerSummaries {
    type View = ContainerSummaryListView;

    fn cache_key(&self) -> String {
        format!(
            "list_container_summaries:{}:{}:{}",
            self.limit,
            self.id.as_deref().unwrap_or_default(),
            self.cursor.as_deref().unwrap_or_default()
        )
    }
}

impl SqlDql for ListContainerSummaries {
    fn build(&self) -> (String, Params) {
        let last_id = Cursor::last_id_or_start(self.cursor.as_deref(), &self.cursor_filters());

        let sql = format!(
            "SELECT c.id, c.code, c.current_weight, c.max_capacity, c.status, \
             (SELECT JSON_ARRAYAGG(JSON_OBJECT( \
              'product_id', ci.product_id, 'product_name', p.name, \
              'quantity', ci.quantity, 'weight', ci.weight)) \
              FROM container_items ci \
              INNER JOIN products p ON p.id = ci.product_id \
              WHERE ci.container_id = c.id) AS manifest_json, \
             (SELECT JSON_ARRAYAGG(JSON_OBJECT( \
              'id', t.id, 'event', t.event, 'description', t.description, \
              'timestamp', t.timestamp)) \
              FROM telemetry_logs t \
              WHERE t.container_id = c.id \
              AND t.id >= COALESCE((SELECT t2.id FROM telemetry_logs t2 \
              WHERE t2.container_id = c.id ORDER BY t2.id DESC LIMIT 1 OFFSET {window}), 0)) AS logs_json, \
             (SELECT COUNT(*) FROM containers WHERE deleted_at IS NULL{total_id}) AS _total \
             FROM containers c \
             WHERE c.id > :last_id AND c.deleted_at IS NULL{page_id} \
             ORDER BY c.id ASC LIMIT :limit",
            window = RECENT_LOGS - 1,
            total_id = self.id_condition(""),
            page_id = self.id_condition("c."),
        );

        let mut values = vec![
            ("last_id".to_owned(), SqlValue::Int(last_id)),
            ("limit".to_owned(), SqlValue::Int(i64::from(self.limit))),
        ];

        if let Some(id) = self.raw_id {
            values.push(("id".to_owned(), SqlValue::Int(id)));
        }

        (sql, values.into())
    }

    fn read(&self, rows: Vec<Row>) -> anyhow::Result<Self::View> {
        let mut items = Vec::with_capacity(self.limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            let manifest_json: Option<String> = Column::of(row, "manifest_json")?;
            let logs_json: Option<String> = Column::of(row, "logs_json")?;

            items.push(ContainerSummaryViewItem {
                container: read_item(row)?,
                manifest: read_manifest(manifest_json.as_deref())?,
                recent_logs: read_logs(logs_json.as_deref())?,
            });

            last_id = Column::of(row, "id")?;
            total = Column::of(row, "_total")?;
        }

        Ok(ContainerSummaryListView {
            next_cursor: Cursor::next(items.len(), self.limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}
