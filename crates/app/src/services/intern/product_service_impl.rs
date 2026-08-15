//! A orquestração de produtos.
//!
//! A moldura de escrita é sempre a mesma: abrir o escopo, conferir cada passo e
//! confirmar no fim. Não há `begin` nem `rollback` escritos — abrir é o primeiro
//! repositório que escreve, e sair do escopo sem confirmar já desfaz tudo. O que
//! sobra no corpo é a regra, e o `?` volta a ser `?`.
//!
//! ## As permissões são privadas
//!
//! Os slugs abaixo são **contrato**: já existem em papéis gravados no banco de
//! quem roda a versão PHP, e renomear qualquer um revoga silenciosamente o
//! acesso de quem o tinha. São `const` privadas porque uma permissão pertence a
//! exatamente um caso de uso — é ele quem a compara com o `UserContext`, e não
//! há segundo lugar no sistema que precise vê-la. O boot as registra chamando
//! `declare_permissions`, sem nunca lê-las.

use portmaster_domain::domain::Product;
use portmaster_domain::table_modules::ProductTM;
use portmaster_infra::query::views::{ProductListView, ProductViewItem};
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::{ProductRepository, ViewCacheRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::metadata::RegisterPermissionCommand;
use crate::commands::product::CreateProductCommand;
use crate::commands::product::DeleteProductCommand;
use crate::commands::product::UpdateProductCommand;
use crate::error::{AppError, ProductError};
use crate::queries::product::GetProductQuery;
use crate::queries::product::ListProductsQuery;
use crate::services::MetadataService;
use crate::services::ProductService;

/// Cadastrar um produto.
const CREATE: &str = "product:create";
/// Remover um produto.
const DELETE: &str = "product:delete";
/// Ler produtos.
const READ: &str = "product:read";
/// Alterar um produto.
const UPDATE: &str = "product:update";

/// O prefixo das listagens deste serviço — é o que uma escrita derruba.
///
/// A invalidação alcança **só** este prefixo. Uma escrita de produto muda o
/// `registered_products` do painel, e o painel não é derrubado por isso: ele
/// vive com o TTL do cache de leitura, que é curto. Derrubar chave de outro
/// serviço faria cada um deles precisar saber quem mais o lê.
const CACHE_GROUP: &str = "product";

/// A implementação, genérica sobre os ports que consome.
///
/// Nada de `Arc<dyn>`: os tipos concretos chegam do provider e o compilador
/// monomorfiza o grafo inteiro. Um caso de uso que não pudesse ser montado seria
/// erro de compilação, não surpresa no primeiro request.
#[derive(Clone)]
pub(crate) struct ProductServiceImpl<R, T, Q, C> {
    /// Persistência de produtos.
    products: R,
    /// As regras de produto.
    product_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
}

impl<R, T, Q, C> ProductServiceImpl<R, T, Q, C> {
    /// Monta o caso de uso.
    pub(crate) const fn new(products: R, product_tm: T, queries: Q, views: C) -> Self {
        Self {
            products,
            product_tm,
            queries,
            views,
        }
    }
}

impl<R, T, Q, C> ProductService for ProductServiceImpl<R, T, Q, C>
where
    R: ProductRepository + Send + Sync,
    T: ProductTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ProductError> {
        for slug in [CREATE, DELETE, READ, UPDATE] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    /// Cria um produto.
    ///
    /// A permissão é conferida na primeira linha, **antes** do banco. O
    /// `TableModule` valida e instancia — o caso de uso **nunca** constrói um
    /// objeto de domínio, senão a regra de validação teria dois donos.
    async fn create(
        &self,
        command: CreateProductCommand,
    ) -> Result<Box<dyn Product>, ProductError> {
        if !command.context.has_permission(CREATE) {
            return Err(AppError::permission_denied(CREATE).into());
        }

        let product = MasterScope::run(|uow| async move {
            let product =
                self.product_tm
                    .create(command.name, command.density, command.risk_class)?;

            self.products.insert(product.as_ref()).await?;

            uow.commit().await?;

            Ok::<_, ProductError>(product)
        })
        .await?;

        // Depois do commit: antes, uma leitura concorrente repovoaria o cache
        // com o estado antigo, ainda visível para ela.
        self.views.invalidate(CACHE_GROUP).await?;

        Ok(product)
    }

    async fn update(
        &self,
        command: UpdateProductCommand,
    ) -> Result<Box<dyn Product>, ProductError> {
        if !command.context.has_permission(UPDATE) {
            return Err(AppError::permission_denied(UPDATE).into());
        }

        let product = MasterScope::run(|uow| async move {
            let Some(existing) = self.products.find_by_id(&command.id).await? else {
                return Err(ProductError::Missing(command.id));
            };

            let updated = self.product_tm.update(
                existing.as_ref(),
                command.name,
                command.density,
                command.risk_class,
            )?;

            self.products.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(product)
    }

    /// Remove um produto — soft-delete.
    ///
    /// Confere a existência antes de apagar: sem isso, remover duas vezes
    /// responderia sucesso na segunda, e o cliente não teria como saber que o id
    /// que ele mandou nunca existiu.
    async fn delete(&self, command: DeleteProductCommand) -> Result<(), ProductError> {
        if !command.context.has_permission(DELETE) {
            return Err(AppError::permission_denied(DELETE).into());
        }

        MasterScope::run(|uow| async move {
            if self.products.find_by_id(&command.id).await?.is_none() {
                return Err(ProductError::Missing(command.id));
            }

            self.products.delete(&command.id).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }

    /// Um produto, direto da consulta — a leitura por id não passa pelo cache.
    async fn get(&self, query: GetProductQuery) -> Result<ProductViewItem, ProductError> {
        if !query.context.has_permission(READ) {
            return Err(AppError::permission_denied(READ).into());
        }

        let dql = dql::get_product(&query.id)?;

        let missing = query.id.clone();

        let view = MasterScope::run(|uow| async move {
            let Some(view) = self.queries.run(dql).await? else {
                return Err(ProductError::Missing(missing));
            };

            uow.commit().await?;

            Ok(view)
        })
        .await?;

        Ok(view)
    }

    /// A listagem, atrás do cache de leitura.
    ///
    /// A chave sai do próprio DQL, que é quem conhece todos os filtros. Antes
    /// era montada aqui, e um filtro novo que ninguém somasse à ela faria duas
    /// consultas diferentes lerem a mesma entrada.
    ///
    /// O cache é consultado **depois** de autorizar: antes, ele entregaria dado
    /// a quem não pode vê-lo, porque o cache não sabe quem está perguntando. Um
    /// valor que não desserializa é tratado como ausência — ele é de um formato
    /// anterior, e recalcular em silêncio evita que um deploy vire incidente.
    async fn list(&self, query: ListProductsQuery) -> Result<ProductListView, ProductError> {
        if !query.context.has_permission(READ) {
            return Err(AppError::permission_denied(READ).into());
        }

        let dql = dql::list_products(query.cursor.clone(), query.limit, query.search.as_deref());
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, ProductError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }
}

/// Os slugs deste serviço, para o teste do catálogo.
///
/// `cfg(test)`: em produção nada além deste arquivo vê um slug, e é isso que se
/// quer. O teste do catálogo precisa somá-los para afirmar as 25 permissões que
/// já existem em papéis gravados, e essa é a única razão de a lista existir.
#[cfg(test)]
pub(crate) const PERMISSIONS: &[&str] = &[CREATE, DELETE, READ, UPDATE];

#[cfg(test)]
#[path = "tests/product_service_impl_test.rs"]
mod tests;
