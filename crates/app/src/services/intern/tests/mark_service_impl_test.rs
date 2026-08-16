//! Os testes de `mark_service_impl`.
//!
//! Este service não confere permissão em método nenhum, e é deliberado: quem o
//! chama é o boot e o fluxo de sessão, nunca um cliente. Os testes fixam isso —
//! se um dia alguém acrescentar a checagem, é aqui que aparece.

use portmaster_domain::error::{FieldError, MarkerError as DomainMarkerError};

use super::*;
use crate::tests::factories::marker_factory::StubMarker;
use crate::tests::factories::marker_group_factory::StubMarkerGroup;
use crate::tests::mocks::marker_group_repository_mock::MockMarkerGroups;
use crate::tests::mocks::marker_group_tm_mock::MockMarkerGroupRules;
use crate::tests::mocks::marker_repository_mock::MockMarkers;
use crate::tests::mocks::marker_tm_mock::MockMarkerRules;

/// O service com os mocks que o teste armou.
fn service(
    marker_rules: MockMarkerRules,
    group_rules: MockMarkerGroupRules,
    markers: MockMarkers,
    groups: MockMarkerGroups,
) -> impl MarkService {
    mark_service(marker_rules, group_rules, markers, groups)
}

/// Um grupo válido é criado pelo table module e registrado.
#[tokio::test]
async fn registrar_grupo_passa_pelo_table_module() {
    let mut group_rules = MockMarkerGroupRules::new();
    group_rules
        .expect_create()
        .times(1)
        .returning(|_| Ok(StubMarkerGroup::boxed("refresh-token")));

    let mut groups = MockMarkerGroups::new();
    groups.expect_register().times(1).returning(|_| Ok(()));

    service(
        MockMarkerRules::new(),
        group_rules,
        MockMarkers::new(),
        groups,
    )
    .register_group(RegisterMarkerGroupCommand {
        slug: "refresh-token".to_owned(),
    })
    .await
    .expect("o boot registra o grupo");
}

/// O slug que o table module recusa não é registrado.
#[tokio::test]
async fn slug_de_grupo_invalido_nao_e_registrado() {
    let mut group_rules = MockMarkerGroupRules::new();
    group_rules.expect_create().times(1).returning(|_| {
        Err(portmaster_domain::error::MetadataError::Validation(vec![
            FieldError::new("slug", "fora do formato lower-kebab"),
        ]))
    });

    let mut groups = MockMarkerGroups::new();
    groups.expect_register().never();

    service(
        MockMarkerRules::new(),
        group_rules,
        MockMarkers::new(),
        groups,
    )
    .register_group(RegisterMarkerGroupCommand {
        slug: "Refresh Token".to_owned(),
    })
    .await
    .expect_err("um slug fora do formato não vira grupo");
}

/// O valor em claro **não** chega ao repositório: o que vai é o digest.
///
/// É a asserção que protege o segredo. O `MarkerTM` reduz o valor à chave, e o
/// repositório só vê o resultado — se um dia o service passasse `command.value`
/// direto, o refresh token cru iria parar no armazenamento.
#[tokio::test]
async fn o_valor_em_claro_nao_chega_ao_repositorio() {
    let mut marker_rules = MockMarkerRules::new();
    marker_rules
        .expect_create()
        .times(1)
        .returning(|group, _, _| Ok(StubMarker::boxed(&group, "digest-do-valor")));

    let mut markers = MockMarkers::new();
    markers
        .expect_put()
        .withf(|marker, ttl| {
            marker.key() == "digest-do-valor" && marker.key() != "o-segredo" && *ttl == 3_600
        })
        .times(1)
        .returning(|_, _| Ok(()));

    service(
        marker_rules,
        MockMarkerGroupRules::new(),
        markers,
        MockMarkerGroups::new(),
    )
    .set(SetMarkerCommand {
        group: "refresh-token".to_owned(),
        value: "o-segredo".to_owned(),
        flag: true,
        ttl_seconds: 3_600,
    })
    .await
    .expect("gravar o marcador não falha");
}

/// Conferir também passa pelo table module, e pela mesma razão.
///
/// Reduzir o valor à chave aqui duplicaria a conversão, e as duas divergiriam no
/// dia em que o hash mudasse.
#[tokio::test]
async fn conferir_pergunta_pelo_digest() {
    let mut marker_rules = MockMarkerRules::new();
    marker_rules
        .expect_create()
        .times(1)
        .returning(|group, _, _| Ok(StubMarker::boxed(&group, "digest-do-valor")));

    let mut markers = MockMarkers::new();
    markers
        .expect_is_valid()
        .withf(|group, key| group == "refresh-token" && key == "digest-do-valor")
        .times(1)
        .returning(|_, _| Ok(true));

    let valid = service(
        marker_rules,
        MockMarkerGroupRules::new(),
        markers,
        MockMarkerGroups::new(),
    )
    .is_valid(GetMarkerQuery {
        group: "refresh-token".to_owned(),
        value: "o-segredo".to_owned(),
    })
    .await
    .expect("conferir não falha");

    assert!(valid);
}

/// Um marcador que o table module recusa não é gravado.
#[tokio::test]
async fn marcador_recusado_nao_e_gravado() {
    let mut marker_rules = MockMarkerRules::new();
    marker_rules.expect_create().times(1).returning(|_, _, _| {
        Err(DomainMarkerError::Validation(vec![FieldError::new(
            "group",
            "grupo não registrado",
        )]))
    });

    let mut markers = MockMarkers::new();
    markers.expect_put().never();

    service(
        marker_rules,
        MockMarkerGroupRules::new(),
        markers,
        MockMarkerGroups::new(),
    )
    .set(SetMarkerCommand {
        group: "grupo-que-nao-existe".to_owned(),
        value: "o-segredo".to_owned(),
        flag: true,
        ttl_seconds: 3_600,
    })
    .await
    .expect_err("um grupo não registrado recusa");
}
