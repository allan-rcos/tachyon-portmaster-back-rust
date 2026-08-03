//! A consulta compilada: o texto e os valores a ligar.
//!
//! Do lado da **escrita** todo SQL é literal `const` — o dinamismo lá seria só
//! interpolar um nome de tabela fixo, e um literal diz mais. Do lado da
//! **leitura** isso não se sustenta: uma listagem tem filtro opcional, e o
//! `WHERE` muda de forma conforme o que chegou na querystring. Daí este
//! construtor mínimo.
//!
//! ## Por que os binds moram em três listas
//!
//! Com placeholder posicional (`?`), a ordem em que os valores são ligados
//! **tem** que ser a ordem em que os `?` aparecem no texto. Guardar tudo numa
//! lista só faria a corretude depender de chamar os métodos na mesma sequência
//! em que as cláusulas são renderizadas — um acoplamento invisível, que quebra
//! calado no dia em que alguém mover uma linha. Separando por cláusula, a
//! concatenação na renderização é o que define a ordem, e não há como errar.
//!
//! ## Por que `LIMIT`/`OFFSET` são literais
//!
//! São inteiros nossos, nunca texto do cliente, e o MySQL rejeita um placeholder
//! de `LIMIT` ligado como string. Renderizá-los direto evita o problema sem
//! abrir nenhuma superfície de injeção.

use std::fmt::Write as _;

/// Um valor a ligar no placeholder.
///
/// O tipo viaja junto porque o `sqlx` liga por tipo: um inteiro enviado como
/// string faz o MariaDB comparar número com texto e ignorar o índice.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Bind {
    /// Um inteiro — id, contagem, índice de enum.
    Int(i64),
    /// Um texto — termo de busca já normalizado.
    Text(String),
}

/// Uma consulta pronta para executar.
///
/// É o que um [`SqlDql`](super::SqlDql) produz e o que o
/// [`QueryRepository`](super::QueryRepository) consome. O repositório nunca monta
/// SQL: ele recebe isto e roda.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlQuery {
    /// O texto, com placeholders posicionais.
    pub(crate) sql: String,
    /// Os valores, na ordem dos placeholders.
    pub(crate) binds: Vec<Bind>,
}

/// Monta um `SELECT`.
///
/// Deliberadamente pequeno: cobre o que as dez consultas de leitura precisam e
/// nada mais. Um construtor de SQL genérico seria uma segunda linguagem para
/// manter, com o dobro de superfície e nenhum ganho — as consultas aqui são
/// conhecidas e finitas.
#[derive(Debug, Default)]
pub(crate) struct Select {
    columns: Vec<String>,
    column_binds: Vec<Bind>,
    from: String,
    joins: Vec<String>,
    join_binds: Vec<Bind>,
    conditions: Vec<String>,
    condition_binds: Vec<Bind>,
    order_by: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl Select {
    /// Começa um `SELECT` sobre uma fonte.
    ///
    /// `source` é a cláusula `FROM` inteira — tabela, alias, ou uma tabela
    /// derivada já parentizada. Nunca vem de fora: é sempre literal do DQL.
    pub(crate) fn from(source: impl Into<String>) -> Self {
        Self {
            from: source.into(),
            ..Self::default()
        }
    }

    /// Acrescenta uma coluna ou expressão à projeção.
    pub(crate) fn column(mut self, expression: impl Into<String>) -> Self {
        self.columns.push(expression.into());
        self
    }

    /// Acrescenta uma expressão de projeção que carrega valores.
    ///
    /// É o caso das sub-consultas de `_total`, que repetem os filtros da página
    /// para contar o mesmo conjunto de onde ela sai.
    pub(crate) fn column_bound(
        mut self,
        expression: impl Into<String>,
        binds: impl IntoIterator<Item = Bind>,
    ) -> Self {
        self.columns.push(expression.into());
        self.column_binds.extend(binds);
        self
    }

    /// Acrescenta um `JOIN` — a cláusula inteira, literal.
    pub(crate) fn join(mut self, clause: impl Into<String>) -> Self {
        self.joins.push(clause.into());
        self
    }

    /// Acrescenta uma condição ao `WHERE`.
    ///
    /// As condições são unidas por `AND`: nenhuma consulta de leitura aqui
    /// precisa de `OR`, e admiti-lo exigiria precedência explícita.
    pub(crate) fn filter(
        mut self,
        expression: impl Into<String>,
        binds: impl IntoIterator<Item = Bind>,
    ) -> Self {
        self.conditions.push(expression.into());
        self.condition_binds.extend(binds);
        self
    }

    /// Ordena por uma expressão.
    pub(crate) fn order_by(mut self, expression: impl Into<String>) -> Self {
        self.order_by.push(expression.into());
        self
    }

    /// Limita a página.
    pub(crate) fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Pula as primeiras linhas.
    pub(crate) fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Renderiza só o texto, para embutir esta consulta em outra.
    ///
    /// Quem embute é responsável por levar os binds junto, por
    /// [`binds`](Self::binds) — daí os dois serem separados.
    pub(crate) fn to_sql(&self) -> String {
        let mut sql = String::with_capacity(128);

        sql.push_str("SELECT ");
        sql.push_str(&self.columns.join(", "));
        sql.push_str(" FROM ");
        sql.push_str(&self.from);

        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join);
        }

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            // Literal, não placeholder: ver a nota no topo do módulo.
            let _ = write!(sql, " LIMIT {limit}");
        }

        if let Some(offset) = self.offset {
            let _ = write!(sql, " OFFSET {offset}");
        }

        sql
    }

    /// Os valores desta consulta, na ordem em que os `?` saem no texto.
    pub(crate) fn binds(&self) -> Vec<Bind> {
        let mut binds = Vec::with_capacity(
            self.column_binds.len() + self.join_binds.len() + self.condition_binds.len(),
        );

        binds.extend(self.column_binds.iter().cloned());
        binds.extend(self.join_binds.iter().cloned());
        binds.extend(self.condition_binds.iter().cloned());

        binds
    }

    /// Fecha a consulta.
    pub(crate) fn build(self) -> SqlQuery {
        SqlQuery {
            sql: self.to_sql(),
            binds: self.binds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn monta_o_select_minimo() {
        let query = Select::from("products").column("id").column("name").build();

        assert_eq!(query.sql, "SELECT id, name FROM products");
        assert!(query.binds.is_empty());
    }

    #[test]
    fn as_clausulas_saem_na_ordem_do_sql() {
        let query = Select::from("containers c")
            .column("c.*")
            .join("LEFT JOIN container_items ci ON ci.container_id = c.id")
            .filter("c.id > ?", [Bind::Int(10)])
            .filter("c.search_code LIKE ?", [Bind::Text("%abc%".into())])
            .order_by("c.id ASC")
            .limit(20)
            .build();

        assert_eq!(
            query.sql,
            "SELECT c.* FROM containers c \
             LEFT JOIN container_items ci ON ci.container_id = c.id \
             WHERE c.id > ? AND c.search_code LIKE ? \
             ORDER BY c.id ASC LIMIT 20"
        );
    }

    #[test]
    fn os_binds_da_projecao_vem_antes_dos_do_where() {
        // A garantia que sustenta o placeholder posicional: o `?` do `_total`
        // aparece antes do `?` do `WHERE` no texto, então o valor dele tem que
        // ser ligado antes. Trocar a ordem faria a contagem filtrar por id e a
        // página filtrar por texto — as duas com o valor da outra.
        let query = Select::from("products p")
            .column("p.*")
            .column_bound(
                "(SELECT COUNT(*) FROM products WHERE search_name LIKE ?) AS _total",
                [Bind::Text("%cimento%".into())],
            )
            .filter("p.id > ?", [Bind::Int(42)])
            .build();

        assert_eq!(
            query.binds,
            vec![Bind::Text("%cimento%".into()), Bind::Int(42)]
        );
    }

    #[test]
    fn a_ordem_de_chamada_nao_altera_a_ordem_dos_binds() {
        // O mesmo teste pelo avesso: declarar o filtro antes da projeção não
        // pode mudar nada, porque quem define a ordem é a renderização.
        let query = Select::from("products p")
            .filter("p.id > ?", [Bind::Int(42)])
            .column("p.*")
            .column_bound(
                "(SELECT COUNT(*) FROM products WHERE name LIKE ?) AS _total",
                [Bind::Text("%cimento%".into())],
            )
            .build();

        assert_eq!(
            query.binds,
            vec![Bind::Text("%cimento%".into()), Bind::Int(42)]
        );
    }
}
