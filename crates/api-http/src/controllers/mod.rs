//! Os controllers: wire ↔ Command.
//!
//! Um controller é a junção dos handlers de um recurso; um handler é o método.
//!
//! Fininhos de propósito. Um handler faz quatro coisas e nada além: confere se
//! há sessão, monta o Command, delega ao caso de uso e mapeia o resultado de
//! volta ao VO. Não há regra de negócio, orquestração nem transação aqui — se
//! aparecer, está no lugar errado.
//!
//! ## O handler é o handler do axum
//!
//! O trait declara os extractors que cada rota consome e devolve um
//! [`ApiResponse`](crate::wire::api_response::ApiResponse) pronto. É uma troca
//! deliberada: o axum entra no contrato, e em compensação o módulo de rotas
//! volta a ser só uma tabela.
//!
//! O que ele tinha antes eram as quatro coisas acima **espalhadas**. Conferir a
//! sessão acontecia na rota, montar a resposta acontecia na rota, escolher o
//! status acontecia na rota — e `POST /manifests/unload-item`, que tem um corpo
//! e devolve um objeto, ocupava dez linhas de encanamento para chamar um método
//! de uma linha. Hoje ocupa uma, e o que ela diz é o caminho e o método.
//!
//! O `self` é por valor porque um controller é um punhado de handles: a rota
//! clona o dela por requisição, que é o que o axum faz com todo handler.
//!
//! ## O 401 é o único status que nasce aqui
//!
//! Falta de sessão é a única coisa que o `app` não tem como saber, porque só
//! esta camada lê o token. Permissão (403), validação (422), ausência (404) e
//! conflito (409) vêm todos do `app`, traduzidos em `ports::error`.
//!
//! ## Cada recurso são três arquivos
//!
//! O **trait** declara os handlers: que extractors cada rota consome e o que ela
//! responde. A **impl** em `intern` é genérica sobre os services que o
//! `AppProvider` entrega, cujos tipos são innomeáveis, e sobre as portas de
//! contexto que ela consome. E o módulo de **rotas** é a tabela: caminho, verbo,
//! método.
//!
//! É a trait que o `ControllersProvider` devolve — `impl Trait`, como todo o
//! resto do grafo.

pub(crate) mod controllers_provider;
pub(crate) mod params;

pub(crate) mod account_controller;
pub(crate) mod account_routes;
pub(crate) mod auth_controller;
pub(crate) mod auth_routes;
pub(crate) mod container_controller;
pub(crate) mod container_routes;
pub(crate) mod manifest_controller;
pub(crate) mod manifest_routes;
pub(crate) mod metadata_controller;
pub(crate) mod metadata_routes;
pub(crate) mod metrics_controller;
pub(crate) mod metrics_routes;
pub(crate) mod product_controller;
pub(crate) mod product_routes;
pub(crate) mod role_controller;
pub(crate) mod role_routes;
pub(crate) mod server_controller;
pub(crate) mod server_routes;
pub(crate) mod user_controller;
pub(crate) mod user_routes;

pub(crate) mod intern;

pub(crate) use controllers_provider::ControllersProvider;
