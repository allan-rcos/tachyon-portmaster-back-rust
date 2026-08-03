//! View → tabela de wire.
//!
//! O mapeamento é **estático** e mora só aqui, que é o dono do wire. Nem a
//! `infra` (que produz a View) nem o `app` (que a repassa) sabem que existe uma
//! tabela FlatBuffers do outro lado.
//!
//! ## Por que uma macro, e não `From` à mão
//!
//! As tabelas geradas guardam todo campo de referência como `Option<T>`, porque
//! no FlatBuffers um campo pode estar ausente do buffer. As Views não têm essa
//! ambiguidade — um produto **tem** nome. O resultado é que quase todo campo do
//! mapeamento é um `Some(...)` em volta de um movimento direto, e escrever isso
//! trinta vezes à mão é onde um `Some(view.name)` vira `Some(view.code)` sem
//! ninguém notar.
//!
//! A macro escreve exatamente o `From` que seria escrito à mão — mesmo runtime,
//! sem lib externa — e deixa visível só o que **não** é óbvio: os campos que
//! precisam de conversão.
//!
//! > **Frunk foi rejeitado.** Daria o mesmo runtime, mas puxaria a lib para a
//! > `infra` (nas Views) **e** para cá, e forçaria a View a espelhar o wire
//! > campo a campo — atrito garantido no primeiro campo computado.

/// Gera o `From<View>` de uma tabela de wire.
///
/// Um campo escrito sozinho vira `Some(view.campo)` — o caso comum. Um campo
/// com `= expressão` usa a expressão, que enxerga o binding nomeado antes do
/// `=>`.
///
/// ```ignore
/// map_view!(ProductViewItem as view => ProductResponse {
///     id,                                        // Some(view.id)
///     name,                                      // Some(view.name)
///     density = view.density,                    // já é f64, sem Option
///     risk_class = risk_class(view.risk_class),  // i32 → enum do wire
/// });
/// ```
///
/// O binding é nomeado pelo chamador (`as view`) por causa da higiene de macro:
/// um identificador criado dentro da macro não seria visível para a expressão
/// que o chamador escreve.
macro_rules! map_view {
    ($view:ty as $binding:ident => $table:ty { $($fields:tt)* }) => {
        impl From<$view> for $table {
            fn from($binding: $view) -> Self {
                let mut table = <$table>::default();
                map_view!(@field table, $binding, $($fields)*);
                table
            }
        }
    };

    // Fim da lista.
    (@field $table:ident, $binding:ident,) => {};

    // Campo com conversão explícita. Precisa vir antes do caso simples, senão o
    // `ident` casaria primeiro e o `= expr` sobraria sem regra.
    (@field $table:ident, $binding:ident, $field:ident = $value:expr $(, $($rest:tt)*)?) => {
        $table.$field = $value;
        $(map_view!(@field $table, $binding, $($rest)*);)?
    };

    // Campo movido direto, embrulhado no `Option` que a tabela pede.
    (@field $table:ident, $binding:ident, $field:ident $(, $($rest:tt)*)?) => {
        $table.$field = Some($binding.$field);
        $(map_view!(@field $table, $binding, $($rest)*);)?
    };
}

#[cfg(test)]
mod tests {
    use crate::wire::tables as fbs;
    use pretty_assertions::assert_eq;

    /// Uma View de mentira, com os três casos que a macro cobre.
    struct Sample {
        id: String,
        name: String,
        density: f64,
        risk_class: i32,
    }

    map_view!(Sample as view => fbs::product::ProductResponse {
        id,
        name,
        density = view.density,
        risk_class = fbs::common::RiskClass::try_from(view.risk_class as u8)
            .unwrap_or(fbs::common::RiskClass::None),
    });

    #[test]
    fn a_macro_embrulha_o_que_a_tabela_pede_e_converte_o_resto() {
        let table = fbs::product::ProductResponse::from(Sample {
            id: "aZ3".into(),
            name: "Cimento".into(),
            density: 1.44,
            risk_class: 2,
        });

        assert_eq!(table.id.as_deref(), Some("aZ3"));
        assert_eq!(table.name.as_deref(), Some("Cimento"));
        assert_eq!(table.density, 1.44);
        assert_eq!(
            table.risk_class,
            fbs::common::RiskClass::Class3FlammableLiquids
        );
    }

    #[test]
    fn um_indice_fora_da_faixa_nao_entra_em_panico() {
        // A View já validou o índice contra o enum do domínio, mas o wire tem o
        // seu próprio conjunto: se um dia divergirem, cair em `None` é melhor do
        // que derrubar a resposta.
        let table = fbs::product::ProductResponse::from(Sample {
            id: "aZ3".into(),
            name: "Cimento".into(),
            density: 1.0,
            risk_class: 99,
        });

        assert_eq!(table.risk_class, fbs::common::RiskClass::None);
    }
}
