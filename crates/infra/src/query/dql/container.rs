//! As consultas de contêiner.

use anyhow::Context;
use portmaster_domain::enums::{ContainerStatus, TelemetryEvent};
use serde_json::Value;
use sqlx::mysql::MySqlRow;

use super::{effective_limit, like, normalized_search};
use crate::entity::encode_id;
use crate::query::cursor::{filters, Cursor, CursorFilters};
use crate::query::row;
use crate::query::sql::{Bind, Select, SqlQuery};
use crate::query::views::{
    CargoItemView, ContainerListView, ContainerSummaryListView, ContainerSummaryViewItem,
    ContainerViewItem, TelemetryLogView,
};
use crate::query::{ContainerListParams, Dql, SqlDql, SummaryListParams};

/// As colunas que a View de contêiner precisa.
const COLUMNS: &str = "c.id, c.code, c.current_weight, c.max_capacity, c.status";

/// Quantos registros de telemetria a janela recente traz.
const RECENT_LOGS: u32 = 10;

/// Um contêiner pelo id.
pub(crate) struct GetContainerDql {
    id: i64,
}

impl GetContainerDql {
    /// Monta a consulta.
    pub(crate) fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Dql for GetContainerDql {
    type View = Option<ContainerViewItem>;
}

impl SqlDql for GetContainerDql {
    fn build(&self) -> SqlQuery {
        Select::from("containers c")
            .column(COLUMNS)
            .filter("c.id = ?", [Bind::Int(self.id)])
            .filter("c.deleted_at IS NULL", [])
            .limit(1)
            .build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        rows.first().map(item).transpose()
    }
}

/// A listagem de contêineres.
pub(crate) struct ListContainersDql {
    params: ContainerListParams,
}

impl ListContainersDql {
    /// Monta a consulta.
    pub(crate) fn new(params: ContainerListParams) -> Self {
        Self { params }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        filters([
            ("limit", self.limit().to_string()),
            (
                "search",
                normalized_search(self.params.search.as_deref()).unwrap_or_default(),
            ),
            (
                "status",
                self.params
                    .status
                    .map(|s| s.as_i32().to_string())
                    .unwrap_or_default(),
            ),
            (
                "status_in",
                self.params
                    .status_in
                    .iter()
                    .map(|s| s.as_i32().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        effective_limit(self.params.limit)
    }

    /// As condições que a página e a contagem têm que aplicar igualmente.
    ///
    /// Uma lista só, consumida duas vezes com prefixos diferentes: é o que
    /// garante que o total descreva o mesmo conjunto que a página percorre. Duas
    /// listas escritas em paralelo divergiriam no primeiro filtro novo.
    fn conditions(&self, alias: &str) -> Vec<(String, Vec<Bind>)> {
        let mut conditions = Vec::new();

        if let Some(term) = normalized_search(self.params.search.as_deref()) {
            conditions.push((
                format!("{alias}search_code LIKE ?"),
                vec![Bind::Text(like(&term))],
            ));
        }

        if let Some(status) = self.params.status {
            conditions.push((
                format!("{alias}status = ?"),
                vec![Bind::Int(status.as_i32().into())],
            ));
        }

        if !self.params.status_in.is_empty() {
            let placeholders = vec!["?"; self.params.status_in.len()].join(", ");
            conditions.push((
                format!("{alias}status IN ({placeholders})"),
                self.params
                    .status_in
                    .iter()
                    .map(|status| Bind::Int(status.as_i32().into()))
                    .collect(),
            ));
        }

        conditions
    }
}

impl Dql for ListContainersDql {
    type View = ContainerListView;
}

impl SqlDql for ListContainersDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        let mut total = Select::from("containers")
            .column("COUNT(*)")
            .filter("deleted_at IS NULL", []);
        for (condition, binds) in self.conditions("") {
            total = total.filter(condition, binds);
        }

        let mut select = Select::from("containers c")
            .column(COLUMNS)
            .column_bound(format!("({}) AS _total", total.to_sql()), total.binds())
            .filter("c.id > ?", [Bind::Int(last_id)])
            .filter("c.deleted_at IS NULL", [])
            .order_by("c.id ASC")
            .limit(limit);

        for (condition, binds) in self.conditions("c.") {
            select = select.filter(condition, binds);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(item(row)?);
            last_id = row::number(row, "id")?;
            total = row::number(row, "_total")?;
        }

        Ok(ContainerListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

/// A listagem de contêineres com carga e telemetria recente.
pub(crate) struct ListContainerSummariesDql {
    params: SummaryListParams,
    id: Option<i64>,
}

impl ListContainerSummariesDql {
    /// Monta a consulta.
    pub(crate) fn new(params: SummaryListParams, id: Option<i64>) -> Self {
        Self { params, id }
    }

    /// Os filtros sob os quais um cursor desta consulta vale.
    fn cursor_filters(&self) -> CursorFilters {
        filters([
            ("limit", self.limit().to_string()),
            ("id", self.params.id.clone().unwrap_or_default()),
        ])
    }

    /// O tamanho da página.
    fn limit(&self) -> u32 {
        effective_limit(self.params.limit)
    }
}

impl Dql for ListContainerSummariesDql {
    type View = ContainerSummaryListView;
}

impl SqlDql for ListContainerSummariesDql {
    fn build(&self) -> SqlQuery {
        let limit = self.limit();
        let last_id =
            Cursor::last_id_or_start(self.params.cursor.as_deref(), &self.cursor_filters());

        // As duas coleções aninhadas viram JSON no próprio banco. A alternativa
        // — uma consulta por contêiner para a carga e outra para os logs —
        // custaria 2n idas ao banco por página.
        let manifest = "(SELECT JSON_ARRAYAGG(JSON_OBJECT( \
                        'product_id', ci.product_id, 'product_name', p.name, \
                        'quantity', ci.quantity, 'weight', ci.weight)) \
                        FROM container_items ci \
                        INNER JOIN products p ON p.id = ci.product_id \
                        WHERE ci.container_id = c.id) AS manifest_json";

        // O timestamp já sai como epoch em ms, na forma que a View guarda. Sair
        // como texto de data obrigaria a hidratação a interpretar formato de
        // data vindo de dentro de um JSON — duas conversões para chegar no mesmo
        // número.
        //
        // A janela dos recentes é uma sub-consulta escalar correlacionada e não
        // uma tabela derivada: o MariaDB não tem LATERAL, então uma derivada não
        // pode enxergar o `c.id` de fora. Limitar pelo id do n-ésimo mais novo
        // traz as mesmas linhas com uma correlação que o motor aceita.
        let logs = format!(
            "(SELECT JSON_ARRAYAGG(JSON_OBJECT( \
             'id', t.id, 'event', t.event, 'description', t.description, \
             'timestamp', CAST(UNIX_TIMESTAMP(t.timestamp) * 1000 AS SIGNED))) \
             FROM telemetry_logs t \
             WHERE t.container_id = c.id \
             AND t.id >= COALESCE((SELECT t2.id FROM telemetry_logs t2 \
             WHERE t2.container_id = c.id ORDER BY t2.id DESC LIMIT 1 OFFSET {}), 0)) AS logs_json",
            RECENT_LOGS - 1
        );

        let mut total = Select::from("containers")
            .column("COUNT(*)")
            .filter("deleted_at IS NULL", []);
        if let Some(id) = self.id {
            total = total.filter("id = ?", [Bind::Int(id)]);
        }

        let mut select = Select::from("containers c")
            .column(COLUMNS)
            .column(manifest)
            .column(logs)
            .column_bound(format!("({}) AS _total", total.to_sql()), total.binds())
            .filter("c.id > ?", [Bind::Int(last_id)])
            .filter("c.deleted_at IS NULL", [])
            .order_by("c.id ASC")
            .limit(limit);

        if let Some(id) = self.id {
            select = select.filter("c.id = ?", [Bind::Int(id)]);
        }

        select.build()
    }

    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View> {
        let limit = self.limit();
        let mut items = Vec::with_capacity(limit as usize);
        let mut total = 0;
        let mut last_id = 0;

        for row in &rows {
            items.push(ContainerSummaryViewItem {
                container: item(row)?,
                manifest: manifest_of(row::opt_text(row, "manifest_json")?.as_deref())?,
                recent_logs: logs_of(row::opt_text(row, "logs_json")?.as_deref())?,
            });
            last_id = row::number(row, "id")?;
            total = row::number(row, "_total")?;
        }

        Ok(ContainerSummaryListView {
            next_cursor: Cursor::next(items.len(), limit, last_id, &self.cursor_filters()),
            total,
            items,
        })
    }
}

/// Uma linha de `containers` como a View a quer.
fn item(row: &MySqlRow) -> anyhow::Result<ContainerViewItem> {
    Ok(ContainerViewItem {
        id: row::id(row, "id")?,
        code: row::text(row, "code")?,
        current_weight: row::real(row, "current_weight")?,
        max_capacity: row::real(row, "max_capacity")?,
        status: row::enum_index(row, "status", ContainerStatus::from_i32, "ContainerStatus")?,
    })
}

/// O manifesto agregado pelo banco.
///
/// `JSON_ARRAYAGG` devolve `NULL` quando não há linha nenhuma — contêiner vazio,
/// que é estado normal e não ausência de dado.
fn manifest_of(json: Option<&str>) -> anyhow::Result<Vec<CargoItemView>> {
    let entries = entries_of(json, "manifest_json")?;
    let mut items = Vec::with_capacity(entries.len());

    for entry in &entries {
        items.push(CargoItemView {
            product_id: encode_id(int_of(entry, "product_id")?),
            product_name: str_of(entry, "product_name")?,
            quantity: float_of(entry, "quantity")?,
            weight: float_of(entry, "weight")?,
        });
    }

    Ok(items)
}

/// A telemetria recente agregada pelo banco.
fn logs_of(json: Option<&str>) -> anyhow::Result<Vec<TelemetryLogView>> {
    let entries = entries_of(json, "logs_json")?;
    let mut logs = Vec::with_capacity(entries.len());

    for entry in &entries {
        let event = i32::try_from(int_of(entry, "event")?).unwrap_or(-1);

        // Um evento gravado que não corresponde a variante nenhuma é descartado,
        // não aproximado. O campo do fio é um enum: não existe valor que
        // signifique "aconteceu algo, mas nenhum destes", então escolher uma
        // variante reportaria um evento que nunca ocorreu. A linha continua em
        // `telemetry_logs` de qualquer forma.
        if TelemetryEvent::from_i32(event).is_none() {
            continue;
        }

        logs.push(TelemetryLogView {
            id: encode_id(int_of(entry, "id")?),
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
fn entries_of(json: Option<&str>, column: &str) -> anyhow::Result<Vec<Value>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_conjunto_de_status_vira_um_in_com_um_placeholder_por_valor() {
        let query = ListContainersDql::new(ContainerListParams {
            status_in: vec![ContainerStatus::Loading, ContainerStatus::Sealed],
            ..ContainerListParams::default()
        })
        .build();

        assert!(
            query.sql.contains("c.status IN (?, ?)"),
            "esperava um placeholder por status: {}",
            query.sql
        );
        assert_eq!(
            query.binds,
            vec![
                // A contagem primeiro — os `?` dela saem antes no texto.
                Bind::Int(1),
                Bind::Int(2),
                // Depois o cursor e os mesmos status, agora na página.
                Bind::Int(0),
                Bind::Int(1),
                Bind::Int(2),
            ]
        );
    }

    #[test]
    fn a_contagem_repete_exatamente_os_filtros_da_pagina() {
        // Se divergirem, o cliente recebe uma página de três itens dizendo que
        // há quatrocentos.
        let dql = ListContainersDql::new(ContainerListParams {
            search: Some("BR-99".into()),
            status: Some(ContainerStatus::InTransit),
            ..ContainerListParams::default()
        });

        let da_pagina: Vec<_> = dql.conditions("c.").into_iter().map(|(_, b)| b).collect();
        let da_contagem: Vec<_> = dql.conditions("").into_iter().map(|(_, b)| b).collect();

        assert_eq!(da_pagina, da_contagem);
    }

    #[test]
    fn a_janela_de_telemetria_e_correlacionada_por_id() {
        // O MariaDB não tem LATERAL: uma tabela derivada não enxergaria `c.id`.
        let query = ListContainerSummariesDql::new(SummaryListParams::default(), None).build();

        assert!(
            query.sql.contains("LIMIT 1 OFFSET 9"),
            "a janela deveria limitar pelo décimo mais novo: {}",
            query.sql
        );
        assert!(
            query.sql.contains("COALESCE("),
            "faltou o caso de menos logs que o teto"
        );
    }

    #[test]
    fn manifesto_vazio_nao_e_erro() {
        // JSON_ARRAYAGG devolve NULL num contêiner sem carga, que é estado
        // normal.
        assert_eq!(manifest_of(None).unwrap(), Vec::new());
    }

    #[test]
    fn o_manifesto_sai_do_json_agregado() {
        let json = r#"[{"product_id":1,"product_name":"Cimento","quantity":2.5,"weight":50.0}]"#;

        assert_eq!(
            manifest_of(Some(json)).unwrap(),
            vec![CargoItemView {
                product_id: encode_id(1),
                product_name: "Cimento".into(),
                quantity: 2.5,
                weight: 50.0,
            }]
        );
    }

    #[test]
    fn evento_desconhecido_e_descartado_e_nao_aproximado() {
        // O campo do fio é um enum: não há valor que signifique "aconteceu algo,
        // mas nenhum destes".
        let json = r#"[{"id":1,"event":0,"description":null,"timestamp":1000},
                       {"id":2,"event":98,"description":null,"timestamp":2000}]"#;

        let logs = logs_of(Some(json)).unwrap();

        assert_eq!(logs.len(), 1, "o evento fora da faixa deveria ter saído");
        assert_eq!(logs[0].event, TelemetryEvent::Load.as_i32());
        assert_eq!(logs[0].timestamp, 1000);
    }
}
