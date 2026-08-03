//! O lado de leitura do CQRS.
//!
//! A escrita e a leitura são assimétricas de propósito. Uma escrita passa pelo
//! `domain`, aplica regra e devolve um objeto de domínio read-only
//! (`Box<dyn User>`). Uma leitura **não tem regra**: é projeção de saída. Ela
//! não instancia objeto de domínio, não passa pelo `domain`, e por isso não
//! precisa de `dyn` nenhum — devolve uma [`views::AccountView`] por valor, plana
//! e monomórfica. A leitura é o lado que **elimina** o `dyn`, não o que o
//! introduz.
//!
//! ## Uma fonte só, e um descritor por consulta
//!
//! Toda consulta percorre o mesmo caminho no repositório: pegar a transação
//! corrente, executar, transformar linhas em View. Só o **quê** muda. Então há
//! **um** [`QueryRepository`] genérico, e o que varia vive num **DQL** — um
//! descritor que carrega os parâmetros, sabe virar consulta do backend atual e
//! sabe hidratar as próprias linhas.
//!
//! É o que torna a troca de backend barata: um dia em que os dados saiam do
//! MongoDB, os DQLs passam a implementar um `MongoDql` e o repositório passa a
//! consumi-lo. As Views, o `app` e o `api-http` não mudam uma linha.
//!
//! ## Por que os DQLs não são exportados
//!
//! Um DQL é detalhe de como esta camada consulta. Se o `app` pudesse construir
//! um, poderia também inventar consultas — e a fronteira que mantém SQL fora do
//! `app` deixaria de existir. O que sai daqui é a [`QueryFactory`]: o `app` diz
//! **o que** quer e com quais parâmetros, nunca **como**.

pub mod views;

pub(crate) mod cursor;
pub(crate) mod dql;
pub(crate) mod row;
pub(crate) mod sql;

use anyhow::Context;
use portmaster_domain::enums::ContainerStatus;
use sqlx::mysql::MySqlRow;

use crate::database::uow::MariadbUnitOfWork;
use crate::entity::decode_id;
use sql::Bind;
use views::{
    AccountView, ContainerListView, ContainerSummaryListView, ContainerViewItem, MetricsView,
    ProductListView, ProductViewItem, RoleListView, RoleViewItem, UserListView,
};

pub use sql::SqlQuery;

/// O limite de página quando o cliente não pede um.
///
/// Vinte é o que o PHP servia; mudar isso mudaria o tamanho de resposta de todo
/// cliente que nunca passou `limit`.
pub const DEFAULT_LIMIT: u32 = 20;

/// Uma consulta de leitura, independente de backend.
///
/// Amarra só o tipo de saída. É o que permite ao [`QueryRepository`] devolver
/// `D::View` sem saber o que `View` é.
pub trait Dql {
    /// O read model que esta consulta produz.
    ///
    /// `Send` porque a View atravessa o `.await` da execução e sai por uma
    /// fronteira de tarefa — o handler que a pediu pode estar em outra thread.
    type View: Send;
}

/// A face SQL de uma consulta — a que vale hoje.
///
/// Um backend novo ganha a sua própria face (`MongoDql`, com `filter`/`options`)
/// e o repositório correspondente passa a consumi-la. Nada disso alcança a View.
pub trait SqlDql: Dql + Send {
    /// Compila a consulta.
    ///
    /// Chamado uma vez por execução, então o DQL monta o SQL a partir dos
    /// filtros que recebeu em vez de guardar um texto pronto.
    fn build(&self) -> SqlQuery;

    /// Transforma as linhas na View.
    ///
    /// É o único lugar que conhece o tipo concreto de saída — o que deixa o
    /// repositório genérico.
    ///
    /// Falha quando uma linha não corresponde ao schema: um índice de enum fora
    /// da faixa, por exemplo. A alternativa seria escolher uma variante por
    /// aproximação, e uma View que reporta `Class1Explosives` porque o valor
    /// gravado não bateu com nada estaria afirmando que a carga é explosiva.
    fn read(&self, rows: Vec<MySqlRow>) -> anyhow::Result<Self::View>;
}

/// Roda uma consulta e devolve a View que ela hidratou.
///
/// A única saída do lado de leitura. O que devolve objeto de domínio é
/// repositório de escrita, não isto.
#[trait_variant::make(Send)]
pub trait QueryRepository {
    /// Executa e devolve a View **por valor** — monomorfizada, sem `Box<dyn>`.
    ///
    /// Um resultado vazio é sucesso com View vazia, nunca um erro: decidir que
    /// "não achei nada" é problema é do chamador, não da execução.
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View>;
}

/// Os filtros de uma listagem paginada por cursor.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`].
    pub limit: Option<u32>,
    /// Termo de busca, ainda como o cliente digitou.
    pub search: Option<String>,
}

/// Os filtros da listagem de contêineres.
#[derive(Debug, Clone, Default)]
pub struct ContainerListParams {
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`].
    pub limit: Option<u32>,
    /// Termo de busca sobre o código, ainda como o cliente digitou.
    pub search: Option<String>,
    /// Restringe a um status.
    pub status: Option<ContainerStatus>,
    /// Restringe a um conjunto de status; vazio não filtra.
    pub status_in: Vec<ContainerStatus>,
}

/// Os filtros da listagem de resumos de contêiner.
#[derive(Debug, Clone, Default)]
pub struct SummaryListParams {
    /// Restringe a um contêiner, em base62.
    pub id: Option<String>,
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`].
    pub limit: Option<u32>,
}

/// Os filtros da listagem de usuários.
///
/// Página e limite, não cursor: é a única consulta administrativa em que saltar
/// para uma página arbitrária é o uso real.
#[derive(Debug, Clone, Default)]
pub struct UserListParams {
    /// Página, começando em 1; ausente ou zero vale 1.
    pub page: Option<u32>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`].
    pub limit: Option<u32>,
}

/// Constrói os descritores de consulta.
///
/// O `app` pede a consulta que quer e recebe algo que só sabe ser executado. Não
/// alcança o SQL, não alcança o cursor, não consegue inventar uma consulta que
/// esta camada não tenha declarado.
///
/// Os métodos por id são falíveis porque um id em base62 pode simplesmente não
/// ser base62 — uma URL inventada. Recusar ali é melhor do que abrir transação e
/// consultar por um número arbitrário.
pub trait QueryFactory {
    /// Um usuário com os papéis dele.
    fn get_account(&self, user_id: &str)
        -> anyhow::Result<impl SqlDql<View = Option<AccountView>>>;

    /// Um contêiner.
    fn get_container(
        &self,
        id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<ContainerViewItem>>>;

    /// Um produto.
    fn get_product(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<ProductViewItem>>>;

    /// Um papel.
    fn get_role(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<RoleViewItem>>>;

    /// A listagem de contêineres.
    fn list_containers(&self, params: ContainerListParams)
        -> impl SqlDql<View = ContainerListView>;

    /// A listagem de contêineres com carga e telemetria recente.
    fn list_container_summaries(
        &self,
        params: SummaryListParams,
    ) -> anyhow::Result<impl SqlDql<View = ContainerSummaryListView>>;

    /// A listagem de produtos.
    fn list_products(&self, params: ListParams) -> impl SqlDql<View = ProductListView>;

    /// A listagem de papéis.
    fn list_roles(&self, params: ListParams) -> impl SqlDql<View = RoleListView>;

    /// A listagem de usuários.
    fn list_users(&self, params: UserListParams) -> impl SqlDql<View = UserListView>;

    /// O painel do pátio.
    fn metrics(&self) -> impl SqlDql<View = MetricsView>;
}

/// A implementação sobre MariaDB.
///
/// Sem estado: a transação vem do escopo da requisição, o que permite ao
/// provider reconstruí-la a cada chamada por custo nenhum.
pub(crate) struct MariadbQueryRepository;

impl MariadbQueryRepository {
    /// Monta o repositório.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl QueryRepository for MariadbQueryRepository {
    async fn run<D: SqlDql>(&self, dql: D) -> anyhow::Result<D::View> {
        let SqlQuery { sql, binds } = dql.build();
        let mut transaction = MariadbUnitOfWork::current().await?;

        // `AssertSqlSafe` é o que o sqlx 0.9 exige de todo SQL montado em tempo
        // de execução — e a exigência é justa: o texto não é mais uma constante
        // que se lê no arquivo. A afirmação se sustenta porque o
        // [`Select`](sql::Select) nunca interpola valor nenhum: tudo que vem de
        // fora entra como [`Bind`], e o único trecho de texto variável são
        // placeholders `?` contados a partir do tamanho de um `Vec`.
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.clone()));
        for bind in binds {
            // Ligar por tipo, não por texto: um id enviado como string faz o
            // MariaDB comparar número com texto e descartar o índice.
            query = match bind {
                Bind::Int(value) => query.bind(value),
                Bind::Text(value) => query.bind(value),
            };
        }

        let rows = query
            .fetch_all(&mut **transaction.as_mut())
            .await
            .with_context(|| format!("falha ao executar a consulta de leitura: {sql}"))?;

        dql.read(rows)
    }
}

/// A implementação da fábrica de DQLs.
pub(crate) struct MariadbQueryFactory;

impl MariadbQueryFactory {
    /// Monta a fábrica.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl QueryFactory for MariadbQueryFactory {
    fn get_account(
        &self,
        user_id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<AccountView>>> {
        Ok(dql::account::GetAccountDql::new(decode_id(user_id)?))
    }

    fn get_container(
        &self,
        id: &str,
    ) -> anyhow::Result<impl SqlDql<View = Option<ContainerViewItem>>> {
        Ok(dql::container::GetContainerDql::new(decode_id(id)?))
    }

    fn get_product(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<ProductViewItem>>> {
        Ok(dql::product::GetProductDql::new(decode_id(id)?))
    }

    fn get_role(&self, id: &str) -> anyhow::Result<impl SqlDql<View = Option<RoleViewItem>>> {
        Ok(dql::role::GetRoleDql::new(decode_id(id)?))
    }

    fn list_containers(
        &self,
        params: ContainerListParams,
    ) -> impl SqlDql<View = ContainerListView> {
        dql::container::ListContainersDql::new(params)
    }

    fn list_container_summaries(
        &self,
        params: SummaryListParams,
    ) -> anyhow::Result<impl SqlDql<View = ContainerSummaryListView>> {
        let id = params.id.as_deref().map(decode_id).transpose()?;

        Ok(dql::container::ListContainerSummariesDql::new(params, id))
    }

    fn list_products(&self, params: ListParams) -> impl SqlDql<View = ProductListView> {
        dql::product::ListProductsDql::new(params)
    }

    fn list_roles(&self, params: ListParams) -> impl SqlDql<View = RoleListView> {
        dql::role::ListRolesDql::new(params)
    }

    fn list_users(&self, params: UserListParams) -> impl SqlDql<View = UserListView> {
        dql::account::ListUsersDql::new(params)
    }

    fn metrics(&self) -> impl SqlDql<View = MetricsView> {
        dql::metrics::MetricsDql::new()
    }
}
