//! O construtor de `SELECT`.
//!
//! Deliberadamente pequeno: cobre o que as dez consultas de leitura precisam e
//! nada mais. Um construtor de SQL genérico seria uma segunda linguagem para
//! manter, com o dobro de superfície e nenhum ganho.

use crate::query::sql::{Bind, SqlQuery};
use std::fmt::Write as _;

/// Monta um `SELECT`.
///
/// Deliberadamente pequeno: cobre o que as dez consultas de leitura precisam e
/// nada mais. Um construtor de SQL genérico seria uma segunda linguagem para
/// manter, com o dobro de superfície e nenhum ganho — as consultas aqui são
/// conhecidas e finitas.
#[derive(Debug, Default)]
pub(crate) struct Select {
    /// As colunas projetadas, na ordem em que saem.
    columns: Vec<String>,
    /// Os binds das colunas — uma expressão projetada pode ter parâmetro.
    column_binds: Vec<Bind>,
    /// A tabela e o seu alias.
    from: String,
    /// As cláusulas de junção, já escritas.
    joins: Vec<String>,
    /// Os binds das junções, que vêm antes dos do `WHERE`.
    join_binds: Vec<Bind>,
    /// As condições do `WHERE`, unidas por `AND`.
    conditions: Vec<String>,
    /// Os binds das condições.
    condition_binds: Vec<Bind>,
    /// A ordenação, na ordem de precedência.
    order_by: Vec<String>,
    /// Teto de linhas, se houver.
    limit: Option<u32>,
    /// Quantas linhas pular — a paginação por página, não a por cursor.
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
    pub(crate) const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Pula as primeiras linhas.
    pub(crate) const fn offset(mut self, offset: u32) -> Self {
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
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "soma de três len() para dimensionar o Vec: estourar usize exigiria mais binds do que cabem na memória"
        )]
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

    /// A garantia que sustenta o placeholder posicional: o `?` do `_total`
    /// aparece antes do `?` do `WHERE` no texto, então o valor dele tem que
    /// ser ligado antes.
    ///
    /// Trocar a ordem faria a contagem filtrar por id e a página filtrar por
    /// texto — as duas com o valor da outra.
    #[test]
    fn os_binds_da_projecao_vem_antes_dos_do_where() {
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

    /// O mesmo teste pelo avesso: declarar o filtro antes da projeção não pode
    /// mudar nada, porque quem define a ordem é a renderização.
    #[test]
    fn a_ordem_de_chamada_nao_altera_a_ordem_dos_binds() {
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
