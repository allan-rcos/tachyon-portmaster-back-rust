//! Os read models: structs planas, na forma do fio.
//!
//! Uma View não tem regra. É saída pura, e por isso não é um objeto de domínio:
//! não tem `Box<dyn>`, não tem vtable, não tem invariante a proteger. Os campos
//! são públicos porque não há nada a encapsular — encapsular um dado que só
//! existe para ser copiado ao fio seria cerimônia sem função.
//!
//! ## Por que já na forma do fio
//!
//! O tipo rico — `DateTime<Utc>`, `ContainerStatus` — existe do lado da escrita,
//! onde há aritmética de calendário e `match` exaustivo a fazer. Do lado da
//! leitura não há: o próximo passo depois da View é serializar. Guardá-la já
//! como `i64` de epoch em ms, `i32` de índice de enum e `String` base62 de id faz
//! o mapeamento View → resposta ser quase identidade, e evita uma conversão que
//! só existiria para ser desfeita.
//!
//! O FlatBuffers não tem tipo de data (timestamp é `long`) nem enum além de
//! `int`. A View adota essa forma porque é a forma de destino — não por
//! preguiça de tipar.
//!
//! ## Listagem: a base e o filho
//!
//! Onde há agrupamento, a **View** é o agregado (cursor da próxima página,
//! total do conjunto filtrado, e o `Vec` de filhos) e o **ViewItem** é o filho.
//! O `Vec` nasce dimensionado ao limite da página: o tamanho é conhecido antes
//! de ler a primeira linha, e realocar no meio da hidratação seria trabalho
//! gratuito.
//!
//! ## Por que derivam serde
//!
//! Não é para o fio: quem serializa a resposta é o `api-http`, e ele tem os
//! próprios tipos. É para o **cache de leitura**, que guarda bytes e não tipos —
//! um cache por View exigiria um cache por tipo, e o que se quer é um só,
//! indiferente ao que passa por ele. O `app` serializa a View para guardá-la e a
//! reconstrói no acerto.

use serde::{Deserialize, Serialize};

/// Um usuário e os papéis dele.
///
/// Serve tanto `GET /account` (o próprio) quanto cada item de `GET /users` — é o
/// mesmo recorte, e duplicá-lo em dois tipos idênticos só criaria a chance de um
/// divergir do outro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountView {
    /// Id em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
    /// Os papéis atribuídos, ordenados por id.
    pub roles: Vec<RoleViewItem>,
}

/// A listagem de usuários.
///
/// Sem cursor nem total: a listagem de usuários pagina por página/limite, não
/// por keyset, porque é a única consulta administrativa em que pular para uma
/// página arbitrária é o uso real.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserListView {
    /// Os usuários da página.
    pub items: Vec<AccountView>,
}

/// Um papel e o tamanho da sua população.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleViewItem {
    /// Id em base62.
    pub id: String,
    /// Nome do papel.
    pub name: String,
    /// Quantos usuários o têm — computado, sem par na tabela.
    pub user_count: i64,
    /// Os slugs de permissão que ele concede.
    pub permissions: Vec<String>,
}

/// A listagem de papéis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleListView {
    /// Os papéis da página.
    pub items: Vec<RoleViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos papéis o filtro alcança ao todo.
    pub total: i64,
}

/// Um produto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductViewItem {
    /// Id em base62.
    pub id: String,
    /// Nome do produto.
    pub name: String,
    /// Densidade, para converter quantidade em peso.
    pub density: f64,
    /// Índice de [`RiskClass`](portmaster_domain::enums::RiskClass).
    pub risk_class: i32,
}

/// A listagem de produtos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductListView {
    /// Os produtos da página.
    pub items: Vec<ProductViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos produtos o filtro alcança ao todo.
    pub total: i64,
}

/// Um contêiner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerViewItem {
    /// Id em base62.
    pub id: String,
    /// Código de identificação no pátio.
    pub code: String,
    /// Peso embarcado agora.
    pub current_weight: f64,
    /// Capacidade máxima.
    pub max_capacity: f64,
    /// Índice de [`ContainerStatus`](portmaster_domain::enums::ContainerStatus).
    pub status: i32,
}

/// A listagem de contêineres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerListView {
    /// Os contêineres da página.
    pub items: Vec<ContainerViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos contêineres o filtro alcança ao todo.
    pub total: i64,
}

/// Uma linha do manifesto de carga.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoItemView {
    /// Id do produto, em base62.
    pub product_id: String,
    /// Nome do produto, para a listagem não exigir uma segunda consulta.
    pub product_name: String,
    /// Quantidade embarcada.
    pub quantity: f64,
    /// Peso correspondente.
    pub weight: f64,
}

/// Um registro de telemetria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryLogView {
    /// Id em base62.
    pub id: String,
    /// Índice de [`TelemetryEvent`](portmaster_domain::enums::TelemetryEvent).
    pub event: i32,
    /// Descrição livre, quando houver.
    pub description: Option<String>,
    /// Epoch em ms.
    pub timestamp: i64,
}

/// Um contêiner com a carga e o histórico recente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerSummaryViewItem {
    /// O contêiner em si.
    pub container: ContainerViewItem,
    /// A carga embarcada agora.
    pub manifest: Vec<CargoItemView>,
    /// Os últimos registros de telemetria, do mais antigo ao mais novo.
    pub recent_logs: Vec<TelemetryLogView>,
}

/// A listagem de resumos de contêiner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerSummaryListView {
    /// Os resumos da página.
    pub items: Vec<ContainerSummaryViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos contêineres o filtro alcança ao todo.
    pub total: i64,
}

/// Quantos contêineres há em cada status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OccupancyView {
    /// Registrados e sem carga.
    pub empty: i64,
    /// Recebendo carga.
    pub loading: i64,
    /// Fechados, aguardando despacho.
    pub sealed: i64,
    /// Despachados.
    pub in_transit: i64,
}

/// O painel do pátio.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct MetricsView {
    /// Contêineres em qualquer status que não `Empty`.
    pub active_containers: i64,
    /// Contêineres registrados.
    pub total_containers: i64,
    /// Peso total embarcado no pátio.
    pub yard_load: f64,
    /// Produtos cadastrados.
    pub registered_products: i64,
    /// A distribuição por status.
    pub occupancy: OccupancyView,
}
