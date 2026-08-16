//! Persistência de carga e telemetria sobre `MariaDB`.

use anyhow::Context;
use chrono::Utc;
use mysql_async::params;
use mysql_async::prelude::Queryable as _;
use portmaster_domain::domain::ManifestCargo;
use portmaster_domain::enums::TelemetryEvent;

use crate::entity::codec::Codec;
use crate::entity::manifest_cargo_entity::ManifestCargoEntity;
use crate::repository::ManifestRepository;
use crate::scope::database::mysql_transaction::MySqlTransaction;

/// Monta o repositório de manifesto.
///
/// Não guarda estado: a transação vem do escopo da tarefa, não de um campo — o
/// que permite ao provider reconstruí-lo a cada chamada por custo praticamente
/// zero.
pub(super) fn manifest_repository<T>(
    transactions: T,
) -> impl ManifestRepository + Sync + Clone + use<T> + 'static
where
    T: MySqlTransaction + Send + Sync + Clone + 'static,
{
    ManifestMariadbRepository { transactions }
}

/// O repositório de manifesto, sobre o `MariaDB`.
#[derive(Clone)]
struct ManifestMariadbRepository<T> {
    /// De onde a transação da tarefa vem.
    transactions: T,
}

impl<T: MySqlTransaction + Send + Sync> ManifestRepository for ManifestMariadbRepository<T> {
    /// A linha de manifesto de um produto num contêiner.
    async fn find_cargo(
        &self,
        container_id: &str,
        product_id: &str,
    ) -> anyhow::Result<Option<Box<dyn ManifestCargo>>> {
        let raw_container = Codec::decode_id(container_id)?;
        let raw_product = Codec::decode_id(product_id)?;
        let mut transaction = self.transactions.transaction().await?;

        let entity: Option<ManifestCargoEntity> = transaction
            .exec_first(
                "SELECT container_id, product_id, quantity, weight, created_at \
                 FROM `container_items` \
                 WHERE container_id = :container_id AND product_id = :product_id",
                params! { "container_id" => raw_container, "product_id" => raw_product },
            )
            .await
            .with_context(|| {
                format!("falha ao buscar a carga de {product_id} no contêiner {container_id}")
            })?;

        Ok(entity.map(|entity| Box::new(entity) as Box<dyn ManifestCargo>))
    }

    /// Upsert num comando só.
    ///
    /// O par (contêiner, produto) é a chave primária, então embarcar de novo o
    /// mesmo produto atualiza a linha em vez de duplicá-la — sem um `SELECT`
    /// antes para descobrir qual dos dois casos é.
    ///
    /// `created_at` fica de fora do `ON DUPLICATE KEY UPDATE` de propósito: a
    /// linha que já existe nasceu quando nasceu, e reescrever esse instante
    /// apagaria desde quando aquele produto está no contêiner.
    async fn upsert_cargo(&self, cargo: &dyn ManifestCargo) -> anyhow::Result<()> {
        let entity = ManifestCargoEntity::from_domain(cargo)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `container_items` \
                 (container_id, product_id, quantity, weight, created_at) \
                 VALUES (:container_id, :product_id, :quantity, :weight, :created_at) \
                 ON DUPLICATE KEY UPDATE quantity = VALUES(quantity), weight = VALUES(weight)",
                params! {
                    "container_id" => entity.raw_container_id(),
                    "product_id" => entity.raw_product_id(),
                    "quantity" => entity.quantity(),
                    "weight" => entity.weight(),
                    "created_at" => entity.created_at().timestamp_millis(),
                },
            )
            .await
            .context("falha ao gravar a linha do manifesto")?;

        Ok(())
    }

    /// `DELETE` de verdade: carga é entidade fraca.
    ///
    /// Não há o que preservar numa linha que só existia enquanto o produto
    /// estava no contêiner.
    async fn delete_cargo(&self, container_id: &str, product_id: &str) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let raw_product = Codec::decode_id(product_id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "DELETE FROM `container_items` \
                 WHERE container_id = :container_id AND product_id = :product_id",
                params! { "container_id" => raw_container, "product_id" => raw_product },
            )
            .await
            .context("falha ao remover a linha do manifesto")?;

        Ok(())
    }

    /// Esvazia o manifesto inteiro de um contêiner.
    async fn clear_manifest(&self, container_id: &str) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "DELETE FROM `container_items` WHERE container_id = :container_id",
                params! { "container_id" => raw_container },
            )
            .await
            .with_context(|| format!("falha ao limpar o manifesto do contêiner {container_id}"))?;

        Ok(())
    }

    /// Registra o evento de embarque ou desembarque.
    ///
    /// O instante vem desta máquina, e não de uma função de tempo do servidor.
    /// Quem sabe quando o evento aconteceu é quem o registra; delegar o carimbo
    /// ao `MariaDB` gravaria quando o `INSERT` chegou lá, e faria a linha
    /// depender de os dois relógios concordarem.
    async fn insert_telemetry(
        &self,
        container_id: &str,
        event: TelemetryEvent,
        description: Option<&str>,
    ) -> anyhow::Result<()> {
        let raw_container = Codec::decode_id(container_id)?;
        let mut transaction = self.transactions.transaction().await?;

        transaction
            .exec_drop(
                "INSERT INTO `telemetry_logs` (container_id, event, description, timestamp) \
                 VALUES (:container_id, :event, :description, :timestamp)",
                params! {
                    "container_id" => raw_container,
                    "event" => event.as_i32(),
                    "description" => description,
                    "timestamp" => Utc::now().timestamp_millis(),
                },
            )
            .await
            .with_context(|| {
                format!("falha ao registrar telemetria do contêiner {container_id}")
            })?;

        Ok(())
    }
}
