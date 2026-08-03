//! Os ports de persistência.
//!
//! Ficam **aqui** e não no `domain` porque quem os consome é o `app`, que
//! conhece esta camada. O `domain` não sabe que existe banco.
//!
//! Todos usam `async fn` nativo em trait, sem `: Send + Sync`: são consumidos
//! por generics, então os auto-traits são derivados no tipo concreto. E todos
//! devolvem `anyhow::Result` — falha de I/O não é regra de negócio, e misturar
//! as duas faria o `app` tratar "o banco caiu" como "os dados estão errados".
//!
//! Nenhum deles abre transação: rodam na transação corrente, que o `app` abriu.

pub(crate) mod mariadb;

use portmaster_domain::container::Container;
use portmaster_domain::enums::TelemetryEvent;
use portmaster_domain::manifest::ManifestCargo;
use portmaster_domain::marker::Marker;
use portmaster_domain::metadata::marker_group::MarkerGroup;
use portmaster_domain::metadata::permission::Permission;
use portmaster_domain::product::Product;
use portmaster_domain::role::Role;
use portmaster_domain::user::User;

/// Persistência de usuários.
#[trait_variant::make(Send)]
pub trait UserRepository {
    /// Se existe ao menos um usuário.
    ///
    /// É o que o bootstrap consulta: um sistema sem usuário nenhum aceita o
    /// primeiro cadastro; com um, recusa.
    async fn has_any(&self) -> anyhow::Result<bool>;

    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn User>>>;

    /// Busca por e-mail, ou `None` se não existe ou foi removido.
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Box<dyn User>>>;

    /// Grava um usuário novo.
    async fn insert(&self, user: &dyn User) -> anyhow::Result<()>;

    /// Atualiza um usuário existente.
    async fn update(&self, user: &dyn User) -> anyhow::Result<()>;

    /// Remove um usuário — soft-delete, porque usuário é entidade forte.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;

    /// Substitui os papéis de um usuário.
    ///
    /// Substitui, não soma: um papel omitido está revogado.
    async fn sync_roles(&self, user_id: &str, role_ids: &[String]) -> anyhow::Result<()>;
}

/// Persistência de papéis.
#[trait_variant::make(Send)]
pub trait RoleRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>>;

    /// Os papéis de um usuário.
    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>>;

    /// Grava um papel novo.
    async fn insert(&self, role: &dyn Role) -> anyhow::Result<()>;

    /// Atualiza um papel existente.
    async fn update(&self, role: &dyn Role) -> anyhow::Result<()>;

    /// Remove um papel — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}

/// Persistência de produtos.
#[trait_variant::make(Send)]
pub trait ProductRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>>;

    /// Grava um produto novo.
    async fn insert(&self, product: &dyn Product) -> anyhow::Result<()>;

    /// Atualiza um produto existente.
    async fn update(&self, product: &dyn Product) -> anyhow::Result<()>;

    /// Remove um produto — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}

/// Persistência de contêineres.
#[trait_variant::make(Send)]
pub trait ContainerRepository {
    /// Busca por id, ou `None` se não existe ou foi removido.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>>;

    /// Grava um contêiner novo.
    async fn insert(&self, container: &dyn Container) -> anyhow::Result<()>;

    /// Atualiza um contêiner existente.
    async fn update(&self, container: &dyn Container) -> anyhow::Result<()>;

    /// Remove um contêiner — soft-delete.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}

/// Persistência de carga e telemetria.
#[trait_variant::make(Send)]
pub trait ManifestRepository {
    /// A linha de manifesto de um produto num contêiner, se existe.
    async fn find_cargo(
        &self,
        container_id: &str,
        product_id: &str,
    ) -> anyhow::Result<Option<Box<dyn ManifestCargo>>>;

    /// Grava ou substitui a linha de manifesto.
    async fn upsert_cargo(&self, cargo: &dyn ManifestCargo) -> anyhow::Result<()>;

    /// Apaga a linha de um produto.
    ///
    /// `DELETE` de verdade: carga é entidade fraca, sem soft-delete.
    async fn delete_cargo(&self, container_id: &str, product_id: &str) -> anyhow::Result<()>;

    /// Apaga o manifesto inteiro de um contêiner.
    async fn clear_manifest(&self, container_id: &str) -> anyhow::Result<()>;

    /// Registra um movimento na telemetria.
    async fn insert_telemetry(
        &self,
        container_id: &str,
        event: TelemetryEvent,
        description: Option<&str>,
    ) -> anyhow::Result<()>;
}

/// Registro de permissões.
///
/// O backing é **cache**, nunca o banco: o catálogo é preenchido em código no
/// boot, é imutável depois disso, e é lido a cada verificação de autorização.
#[trait_variant::make(Send)]
pub trait PermissionRepository {
    /// Registra uma permissão. Idempotente por slug.
    async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()>;

    /// Todos os slugs registrados.
    async fn all(&self) -> anyhow::Result<Vec<String>>;

    /// Se um slug foi registrado.
    async fn has(&self, slug: &str) -> anyhow::Result<bool>;
}

/// Registro de grupos de marcador.
#[trait_variant::make(Send)]
pub trait MarkerGroupRepository {
    /// Registra um grupo. Idempotente por slug.
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()>;

    /// Se um slug foi registrado.
    async fn has(&self, slug: &str) -> anyhow::Result<bool>;
}

/// Marcadores booleanos com prazo.
///
/// A `infra` nunca sabe o que uma marca significa — só que é um booleano com
/// validade, num grupo conhecido.
#[trait_variant::make(Send)]
pub trait MarkerRepository {
    /// Grava a marca, aplicando as regras de transição.
    async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()>;

    /// Se a marca está válida.
    ///
    /// Marca inexistente, expirada ou desligada respondem igual: `false`. Quem
    /// pergunta quer saber se pode seguir, não por que não pode.
    async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool>;
}
