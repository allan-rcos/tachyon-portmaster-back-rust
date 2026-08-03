//! O marcador: um booleano com prazo, num grupo.
//!
//! Deliberadamente **agnóstico**. Este caso de uso não sabe o que uma marca
//! significa — não sabe o que é sessão, nem refresh token, nem JWT. Ele marca um
//! valor num grupo e responde se a marca vale.
//!
//! Quem dá sentido a isso é o `api-http`, que usa o grupo `refresh-token` e sabe
//! que o valor em claro é o token que ele emitiu. Essa separação é o que mantém
//! `domain`, `app` e `infra` sem nenhum conceito de autenticação por token: no
//! dia em que a sessão virar outra coisa, o grupo muda de nome e nada aqui é
//! tocado.
//!
//! ## O valor em claro não é guardado
//!
//! O TableModule reduz o valor a um digest antes de virar marcador. O que fica
//! em memória é o hash — quem lesse o cache não conseguiria reconstruir os
//! tokens vivos.

use portmaster_domain::marker::MarkerTM;
use portmaster_infra::repository::MarkerRepository;

use crate::error::AppError;

/// Marcar um valor.
#[derive(Debug, Clone)]
pub struct SetMarkerCommand {
    /// O grupo, que precisa estar registrado.
    pub group: String,
    /// O valor em claro — reduzido a digest antes de ser guardado.
    pub value: String,
    /// Ligar ou desligar a marca.
    pub flag: bool,
    /// Por quanto tempo a marca vale.
    pub ttl_seconds: u64,
}

/// Consultar uma marca.
#[derive(Debug, Clone)]
pub struct GetMarkerQuery {
    /// O grupo.
    pub group: String,
    /// O valor em claro.
    pub value: String,
}

/// A primitiva de marcação.
#[trait_variant::make(Send)]
pub trait MarkUseCase {
    /// Marca um valor, aplicando as regras de transição do grupo.
    async fn set(&self, command: SetMarkerCommand) -> Result<(), AppError>;

    /// Se a marca vale agora.
    ///
    /// Marca inexistente, expirada ou desligada respondem igual: `false`. Quem
    /// pergunta quer saber se pode seguir, não por que não pode — e distinguir
    /// os casos diria a quem tentasse adivinhar quais valores já existiram.
    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MarkUseCaseImpl<T, R> {
    marker_tm: T,
    markers: R,
}

impl<T, R> MarkUseCaseImpl<T, R> {
    /// Monta o caso de uso.
    pub(crate) fn new(marker_tm: T, markers: R) -> Self {
        Self { marker_tm, markers }
    }
}

impl<T: MarkerTM + Send + Sync, R: MarkerRepository + Send + Sync> MarkUseCase
    for MarkUseCaseImpl<T, R>
{
    async fn set(&self, command: SetMarkerCommand) -> Result<(), AppError> {
        let marker = self
            .marker_tm
            .create(command.group, &command.value, command.flag)?;

        // As regras de transição — remarcar o que já vale, revalidar o que foi
        // invalidado — são da `infra`, que é dona do estado. Reimplementá-las
        // aqui daria ao sistema duas opiniões sobre a mesma coisa.
        self.markers
            .put(marker.as_ref(), command.ttl_seconds)
            .await?;

        Ok(())
    }

    async fn is_valid(&self, query: GetMarkerQuery) -> Result<bool, AppError> {
        // Cria um marcador só para chegar ao digest: é o TableModule que sabe
        // reduzir o valor em claro à chave, e duplicar essa conversão aqui faria
        // as duas divergirem no dia em que o hash mudasse.
        let marker = self.marker_tm.create(query.group, &query.value, false)?;

        Ok(self.markers.is_valid(marker.group(), marker.key()).await?)
    }
}
