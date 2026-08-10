//! Persistência de carga e telemetria sobre `MariaDB`.

use anyhow::Context;
use portmaster_domain::domain::ManifestCargo;
use portmaster_domain::enums::TelemetryEvent;

use crate::entity::codec::Codec;
use crate::entity::manifest_cargo_entity::ManifestCargoEntity;
use crate::repository::ManifestRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// A linha de manifesto de um produto num contêiner.
const FIND_CARGO: &str = "SELECT container_id, product_id, quantity, weight, created_at \
                          FROM `container_items` WHERE container_id = ? AND product_id = ?";

/// Upsert num comando só: o par (contêiner, produto) é a chave primária, então
/// embarcar de novo o mesmo produto atualiza a linha em vez de duplicá-la — sem
/// um `SELECT` antes para descobrir qual dos dois casos é.
const UPSERT_CARGO: &str =
    "INSERT INTO `container_items` (container_id, product_id, quantity, weight) \
     VALUES (?, ?, ?, ?) \
     ON DUPLICATE KEY UPDATE quantity = VALUES(quantity), weight = VALUES(weight)";

/// `DELETE` de verdade: carga é entidade fraca. Não há o que preservar numa linha
/// que só existia enquanto o produto estava no contêiner.
const DELETE_CARGO: &str =
    "DELETE FROM `container_items` WHERE container_id = ? AND product_id = ?";

/// Esvazia o manifesto inteiro de um contêiner.
const CLEAR_MANIFEST: &str = "DELETE FROM `container_items` WHERE container_id = ?";

/// Registra o evento de embarque ou desembarque.
///
/// Usa `UTC_TIMESTAMP()` e não `NOW()`: o `NOW()` devolve o fuso da **sessão**,
/// e um horário de telemetria que depende de como a conexão foi aberta não é um
/// horário. A sessão do pool já é fixada em `+00:00` quando ele é aberto, o que
/// faria os dois coincidirem — mas
/// escrever o que se quer dizer é o que mantém a linha correta se alguém um dia
/// rodar este SQL por outro caminho.
const INSERT_TELEMETRY: &str =
    "INSERT INTO `telemetry_logs` (container_id, event, description, timestamp) \
     VALUES (?, ?, ?, UTC_TIMESTAMP())";

/// O repositório de manifesto.
#[derive(Clone)]
pub struct ManifestMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T> ManifestMariadbRepository<T> {
    /// Monta o repositório.
    pub(crate) const fn new(transactions: T) -> Self {
        Self { transactions }
    }
}

impl<T: MySqlTransaction + Send + Sync> ManifestRepository for ManifestMariadbRepository<T> {
    async fn find_cargo(
        &self,
        container_id: &str,
        product_id: &str,
    ) -> anyhow::Result<Option<Box<dyn ManifestCargo>>> {
        let raw_container = Codec::decode_id(container_id)?;
        let raw_product = Codec::decode_id(product_id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<ManifestCargoEntity> = sqlx::query_as(FIND_CARGO)
            .bind(raw_container)
            .bind(raw_product)
            .fetch_optional(&mut **transaction)
            .await
            .with_context(|| {
                format!("falha ao buscar a carga de {product_id} no contêiner {container_id}")
            })?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn ManifestCargo>))
    }

    async fn upsert_cargo(&self, cargo: &dyn ManifestCargo) -> anyhow::Result<()> {
        let entity = ManifestCargoEntity::from_domain(cargo)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(UPSERT_CARGO)
            .bind(entity.raw_container_id())
            .bind(entity.raw_product_id())
            .bind(entity.quantity())
            .bind(entity.weight())
            .execute(&mut **transaction)
            .await
            .context("falha ao gravar a linha do manifesto")?;

        Ok(())
    }

    async fn delete_cargo(&self, container_id: &str, product_id: &str) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let raw_product = Codec::decode_id(product_id)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(DELETE_CARGO)
            .bind(raw_container)
            .bind(raw_product)
            .execute(&mut **transaction)
            .await
            .context("falha ao remover a linha do manifesto")?;

        Ok(())
    }

    async fn clear_manifest(&self, container_id: &str) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(CLEAR_MANIFEST)
            .bind(raw_container)
            .execute(&mut **transaction)
            .await
            .with_context(|| format!("falha ao limpar o manifesto do contêiner {container_id}"))?;

        Ok(())
    }

    async fn insert_telemetry(
        &self,
        container_id: &str,
        event: TelemetryEvent,
        description: Option<&str>,
    ) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let mut transaction = self.transactions.transaction().await?;

        sqlx::query(INSERT_TELEMETRY)
            .bind(raw_container)
            .bind(event.as_i32())
            .bind(description)
            .execute(&mut **transaction)
            .await
            .with_context(|| {
                format!("falha ao registrar telemetria do contêiner {container_id}")
            })?;

        Ok(())
    }
}
