//! O formato de fio, e os dois processos que o produzem.
//!
//! Uma requisição chega em FlatBuffers ou JSON e uma resposta sai em um dos dois,
//! negociado por `Content-Type`/`Accept`. Traduzir entre esses formatos são na
//! verdade **duas responsabilidades diferentes**, e mantê-las separadas é o que
//! permite acrescentar um terceiro formato sem reescrever nada:
//!
//! * **Serializar/desserializar** ([`negotiate`]) não depende de *qual* objeto
//!   está em jogo. Todo tipo FlatBuffers desserializa do mesmo jeito, e todo tipo
//!   serde também. Cada `Strategy` conhece um formato e nada sobre o payload.
//! * **Construir** ([`factory`]) depende dos dados e não do formato. O handler
//!   sabe o que responder, mas não sabe o que foi negociado — então entrega os
//!   dados a uma factory, que se passa à strategy, que conduz a construção do
//!   tipo certo. Todas devolvem a mesma coisa: o corpo da resposta.
//!
//! Os schemas `.fbs` são a fonte compartilhada com o cliente e não são alterados
//! por nada disto. Os tipos FlatBuffers em [`fbs`] são gerados a partir deles no
//! build; os tipos JSON são estruturas irmãs, espelhando as mesmas tabelas.

pub(crate) mod http;
#[macro_use]
pub(crate) mod map;
pub(crate) mod negotiate;
pub(crate) mod view;

/// Tipos FlatBuffers gerados pelo planus a partir dos schemas, no build.
///
/// O módulo é gerado inteiro — não editar aqui, e sim no `.fbs` correspondente.
#[allow(
    dead_code,
    unsafe_code,
    clippy::all,
    clippy::pedantic,
    unused_imports,
    missing_docs,
    rustdoc::all
)]
pub(crate) mod fbs {
    include!(concat!(env!("OUT_DIR"), "/wire_generated.rs"));
}

/// As tabelas do wire, sem o aninhamento de namespace do gerador.
///
/// O planus reproduz `API.Fbs.Account` como `api::fbs::account`, o que faria
/// todo handler escrever `fbs::api::fbs::account::…`. O apelido encurta isso
/// para `tables::account::…` sem esconder de onde vem.
pub(crate) use fbs::api::fbs as tables;

// As tabelas que chegam como corpo de requisição. Cada uma precisa da ligação
// explícita com o seu tipo `Ref`, porque o planus não a expõe por trait.
use tables::account::{AccountPasswordChangeRequest, AccountPasswordChangeRequestRef};
use tables::account::{AccountUpdateRequest, AccountUpdateRequestRef};
use tables::admin::{RoleCreateRequest, RoleCreateRequestRef};
use tables::admin::{RolePermissionsUpdateRequest, RolePermissionsUpdateRequestRef};
use tables::admin::{UserAdminPasswordResetRequest, UserAdminPasswordResetRequestRef};
use tables::admin::{UserCreateRequest, UserCreateRequestRef};
use tables::admin::{UserUpdateRequest, UserUpdateRequestRef};
use tables::auth::{LoginRequest, LoginRequestRef, SetupRequest, SetupRequestRef};
use tables::container::{ContainerCreateRequest, ContainerCreateRequestRef};
use tables::container::{ContainerUpdateRequest, ContainerUpdateRequestRef};
use tables::manifest::{LoadItemRequest, LoadItemRequestRef};
use tables::manifest::{UnloadItemRequest, UnloadItemRequestRef};
use tables::product::{ProductCreateRequest, ProductCreateRequestRef};
use tables::product::{ProductUpdateRequest, ProductUpdateRequestRef};

use crate::wire_request;

wire_request!(LoginRequest, LoginRequestRef);
wire_request!(SetupRequest, SetupRequestRef);
wire_request!(AccountUpdateRequest, AccountUpdateRequestRef);
wire_request!(
    AccountPasswordChangeRequest,
    AccountPasswordChangeRequestRef
);
wire_request!(ProductCreateRequest, ProductCreateRequestRef);
wire_request!(ProductUpdateRequest, ProductUpdateRequestRef);
wire_request!(ContainerCreateRequest, ContainerCreateRequestRef);
wire_request!(ContainerUpdateRequest, ContainerUpdateRequestRef);
wire_request!(LoadItemRequest, LoadItemRequestRef);
wire_request!(UnloadItemRequest, UnloadItemRequestRef);
wire_request!(UserCreateRequest, UserCreateRequestRef);
wire_request!(UserUpdateRequest, UserUpdateRequestRef);
wire_request!(
    UserAdminPasswordResetRequest,
    UserAdminPasswordResetRequestRef
);
wire_request!(RoleCreateRequest, RoleCreateRequestRef);
wire_request!(
    RolePermissionsUpdateRequest,
    RolePermissionsUpdateRequestRef
);
