//! Os mapeamentos concretos: o que o `app` devolve → o que sai no fio.
//!
//! Duas origens, dois formatos de saída, um destino:
//!
//! * **Views** (leitura) já chegam *wire-shaped* — id em base62, enum como
//!   índice, timestamp em epoch de ms. O mapeamento é quase identidade.
//! * **Objetos de domínio** (escrita) chegam com tipo rico. Aqui eles perdem o
//!   que a apresentação não publica: um `User` tem `password_hash`, e a tabela
//!   de wire não tem onde pôr isso — o que é a garantia, não o esquecimento.
//!
//! ## Onde os tipos não batem
//!
//! Três lugares, e os três estão explícitos abaixo:
//!
//! * **Contagens**: as Views usam `i64` (é o que `COUNT(*)` devolve); as tabelas
//!   usam `int32`, porque foi o que o `.fbs` fixou. A conversão satura em vez de
//!   truncar — um total absurdo aparece como absurdo, não como negativo.
//! * **Enums**: a View carrega o índice; o wire tem o seu próprio enum. Um
//!   índice fora da faixa cai no valor neutro em vez de derrubar a resposta.
//! * **Timestamp**: a View guarda epoch em ms; `TelemetryLogItem.timestamp` é
//!   `string` no schema. Sai em RFC 3339, que é o que um cliente consegue
//!   interpretar sem saber a convenção de quem escreveu.

use chrono::{DateTime, Utc};
use portmaster_app::domain::{Container, Product, User};
use portmaster_app::views::{
    AccountView, CargoItemView, ContainerListView, ContainerSummaryListView,
    ContainerSummaryViewItem, ContainerViewItem, MetricsView, OccupancyView, ProductListView,
    ProductViewItem, RoleListView, RoleViewItem, TelemetryLogView, UserListView,
};

use super::tables as fbs;

// --- Conversões que não são movimento direto --------------------------------

/// Uma contagem, saturada na faixa que o wire comporta.
fn count(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// O índice de classe de risco, no enum do wire.
fn risk_class(index: i32) -> fbs::common::RiskClass {
    u8::try_from(index)
        .ok()
        .and_then(|index| fbs::common::RiskClass::try_from(index).ok())
        .unwrap_or(fbs::common::RiskClass::None)
}

/// O índice de status, no enum do wire.
fn container_status(index: i32) -> fbs::common::ContainerStatus {
    u8::try_from(index)
        .ok()
        .and_then(|index| fbs::common::ContainerStatus::try_from(index).ok())
        .unwrap_or(fbs::common::ContainerStatus::Empty)
}

/// O índice de evento, no enum do wire.
fn telemetry_event(index: i32) -> fbs::common::TelemetryEvent {
    u8::try_from(index)
        .ok()
        .and_then(|index| fbs::common::TelemetryEvent::try_from(index).ok())
        .unwrap_or(fbs::common::TelemetryEvent::Load)
}

/// Epoch em ms → RFC 3339.
fn timestamp(epoch_ms: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp_millis(epoch_ms).map(|at| at.to_rfc3339())
}

/// Converte uma lista inteira.
fn each<S, T: From<S>>(items: Vec<S>) -> Option<Vec<T>> {
    Some(items.into_iter().map(T::from).collect())
}

// --- Leitura: View → tabela -------------------------------------------------

map_view!(RoleViewItem as view => fbs::account::RoleResponse {
    id,
    name,
    permissions,
    user_count = count(view.user_count),
});

map_view!(AccountView as view => fbs::account::AccountProfileResponse {
    id,
    name,
    email,
    roles = each(view.roles),
});

map_view!(AccountView as view => fbs::admin::UserAdminResponse {
    id,
    name,
    email,
    roles = each(view.roles),
});

map_view!(UserListView as view => fbs::admin::UserListResponse {
    data = each(view.items),
});

map_view!(RoleListView as view => fbs::admin::RoleListResponse {
    next_cursor = view.next_cursor,
    data = each(view.items),
    total = count(view.total),
});

map_view!(ProductViewItem as view => fbs::product::ProductResponse {
    id,
    name,
    density = view.density,
    risk_class = risk_class(view.risk_class),
});

map_view!(ProductListView as view => fbs::product::ProductListResponse {
    next_cursor = view.next_cursor,
    data = each(view.items),
    total = count(view.total),
});

map_view!(ContainerViewItem as view => fbs::container::ContainerResponse {
    id,
    code,
    current_weight = view.current_weight,
    max_capacity = view.max_capacity,
    status = container_status(view.status),
});

map_view!(ContainerListView as view => fbs::container::ContainerListResponse {
    next_cursor = view.next_cursor,
    data = each(view.items),
    total = count(view.total),
});

map_view!(CargoItemView as view => fbs::container::CargoManifestItem {
    product_id,
    product_name,
    quantity = view.quantity,
    weight = view.weight,
});

map_view!(TelemetryLogView as view => fbs::container::TelemetryLogItem {
    id,
    description = view.description,
    event = telemetry_event(view.event),
    timestamp = timestamp(view.timestamp),
});

map_view!(ContainerSummaryViewItem as view => fbs::container::ContainerSummaryResponse {
    container = Some(Box::new(view.container.into())),
    manifest = each(view.manifest),
    recent_logs = each(view.recent_logs),
});

map_view!(ContainerSummaryListView as view => fbs::container::ContainerSummaryListResponse {
    next_cursor = view.next_cursor,
    data = each(view.items),
    total = count(view.total),
});

map_view!(OccupancyView as view => fbs::metrics::OccupancyDivision {
    empty = count(view.empty),
    loading = count(view.loading),
    sealed = count(view.sealed),
    in_transit = count(view.in_transit),
});

map_view!(MetricsView as view => fbs::metrics::MetricsResponse {
    active_containers = count(view.active_containers),
    total_containers = count(view.total_containers),
    yard_load = view.yard_load,
    registered_products = count(view.registered_products),
    occupancy_division = Some(Box::new(view.occupancy.into())),
});

/// Os slugs registrados, na tabela de metadado.
///
/// O `id` é posicional — um handle de consulta, como o schema descreve, e não
/// uma chave estável. O registro é um catálogo em memória preenchido no boot:
/// não há id de banco para publicar, e inventar um sugeriria uma estabilidade
/// que ele não tem.
pub(crate) fn permission_list(slugs: Vec<String>) -> fbs::metadata::PermissionListResponse {
    fbs::metadata::PermissionListResponse {
        data: Some(
            slugs
                .into_iter()
                .enumerate()
                .map(|(index, slug)| fbs::metadata::MetadataItemResponse {
                    id: count(index as i64),
                    slug: Some(slug),
                })
                .collect(),
        ),
    }
}

// --- Escrita: objeto de domínio → tabela ------------------------------------

/// Um usuário no formato enxuto do login.
pub(crate) fn login_user_of(user: &dyn User) -> fbs::auth::User {
    fbs::auth::User {
        id: Some(user.id().to_owned()),
        name: Some(user.name().to_owned()),
        email: Some(user.email().to_owned()),
    }
}

/// Um produto.
pub(crate) fn product_of(product: &dyn Product) -> fbs::product::ProductResponse {
    fbs::product::ProductResponse {
        id: Some(product.id().to_owned()),
        name: Some(product.name().to_owned()),
        density: product.density(),
        risk_class: risk_class(product.risk_class().as_i32()),
    }
}

/// Um contêiner.
pub(crate) fn container_of(container: &dyn Container) -> fbs::container::ContainerResponse {
    fbs::container::ContainerResponse {
        id: Some(container.id().to_owned()),
        code: Some(container.code().to_owned()),
        current_weight: container.current_weight(),
        max_capacity: container.max_capacity(),
        status: container_status(container.status().as_i32()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_contagem_satura_em_vez_de_truncar() {
        // Um `COUNT(*)` acima de 2^31 é absurdo, mas truncá-lo produziria um
        // negativo — que o cliente exibiria como se fosse um dado.
        assert_eq!(count(i64::from(i32::MAX) + 1), i32::MAX);
        assert_eq!(count(7), 7);
    }

    #[test]
    fn um_indice_de_enum_fora_da_faixa_cai_no_neutro() {
        assert_eq!(risk_class(99), fbs::common::RiskClass::None);
        assert_eq!(container_status(-1), fbs::common::ContainerStatus::Empty);
        assert_eq!(
            risk_class(2),
            fbs::common::RiskClass::Class3FlammableLiquids
        );
    }

    #[test]
    fn o_timestamp_sai_interpretavel() {
        // O schema pede string; RFC 3339 é o que um cliente lê sem conhecer a
        // convenção de quem escreveu.
        assert_eq!(timestamp(0).as_deref(), Some("1970-01-01T00:00:00+00:00"));
    }

    #[test]
    fn a_view_de_produto_atravessa_inteira() {
        let table = fbs::product::ProductResponse::from(ProductViewItem {
            id: "aZ3".into(),
            name: "Cimento".into(),
            density: 1.44,
            risk_class: 2,
        });

        assert_eq!(table.id.as_deref(), Some("aZ3"));
        assert_eq!(table.density, 1.44);
        assert_eq!(
            table.risk_class,
            fbs::common::RiskClass::Class3FlammableLiquids
        );
    }

    #[test]
    fn a_listagem_leva_cursor_e_total() {
        let table = fbs::product::ProductListResponse::from(ProductListView {
            items: vec![ProductViewItem {
                id: "aZ3".into(),
                name: "Cimento".into(),
                density: 1.0,
                risk_class: 0,
            }],
            next_cursor: Some("abc".into()),
            total: 42,
        });

        assert_eq!(table.next_cursor.as_deref(), Some("abc"));
        assert_eq!(table.total, 42);
        assert_eq!(table.data.map(|d| d.len()), Some(1));
    }

    #[test]
    fn a_lista_de_permissoes_numera_por_posicao() {
        let table = permission_list(vec!["product:read".into(), "product:create".into()]);
        let data = table.data.expect("a lista tem itens");

        assert_eq!(data[0].id, 0);
        assert_eq!(data[0].slug.as_deref(), Some("product:read"));
        assert_eq!(data[1].id, 1);
    }
}
