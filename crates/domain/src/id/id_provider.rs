//! Quem serve os geradores de id.
//!
//! Três sabores com propósitos diferentes, e um só deles guardado. Este arquivo
//! decide qual — como cada um emite é assunto de `intern/`.

use std::sync::{PoisonError, RwLock};

use crate::config::DomainSecrets;
use crate::id::intern::nano_id_generator::nano_id_generator;
use crate::id::intern::xid_generator::xid_generator;
use crate::id::{DatabaseIdGenerator, RandomIdGenerator, SequentialIdGenerator};

#[cfg(feature = "id-snowflake")]
use crate::id::intern::snowflake_id_generator::{snowflake_id_generator, SnowflakeIdGenerator};

/// O gerador de identidade de entidade do processo.
///
/// Guardado porque precisa ser um só: dois geradores com a mesma identidade
/// emitem o mesmo id, e o porquê aritmético está em [`snowflake_id_generator`].
///
/// `RwLock` porque quem o constrói é configuração, e configuração se troca.
#[cfg(feature = "id-snowflake")]
static DATABASE: RwLock<Option<SnowflakeIdGenerator>> = RwLock::new(None);

/// Os geradores de id, um por sabor.
///
/// Só o de identidade de entidade é guardado. Os outros dois não têm estado a
/// compartilhar, então duas chamadas devolvem coisas indistinguíveis.
pub(crate) struct IdProvider;

impl IdProvider {
    /// Fixa a identidade de deploy, e larga o gerador que estava em uso.
    ///
    /// Instalar de novo troca. Fazê-lo com o processo no ar é operação
    /// consciente: a identidade nova precisa ser de fato nova, senão o processo
    /// passa a emitir ids que se sobrepõem aos de quem já usa aquele par. O
    /// lugar normal de chamar isto é o boot.
    pub(crate) fn install(secrets: DomainSecrets) {
        #[cfg(feature = "id-snowflake")]
        Self::replace(snowflake_id_generator(
            secrets.cluster_id,
            secrets.server_id,
        ));
    }

    /// O gerador de identidade de entidade do processo.
    ///
    /// Sem identidade instalada, vale a instância zero de
    /// [`DomainSecrets::default`] — a mesma que o ambiente sem
    /// `APP_CLUSTER_ID`/`APP_SERVER_ID` já produziria.
    ///
    /// Não é servido pelo [`DomainProvider`](crate::DomainProvider), e é o
    /// único que não é: quem nomeia uma linha é o `TableModule`, e um gerador
    /// destes fora do crate permitiria gravar uma sem passar pela regra que a
    /// valida.
    ///
    /// Qual impl atende é **feature de compilação**, resolvida no build.
    pub(crate) fn database() -> impl DatabaseIdGenerator + Send + Sync + Clone + use<> + 'static {
        #[cfg(feature = "id-snowflake")]
        Self::shared()
    }

    /// O gerador de `request_id`.
    pub(crate) fn sequential() -> impl SequentialIdGenerator + use<> {
        xid_generator()
    }

    /// O gerador de id opaco, para o refresh token.
    pub(crate) fn random() -> impl RandomIdGenerator + use<> {
        nano_id_generator()
    }

    /// Troca o gerador guardado, seja qual for o que estava lá.
    ///
    /// Lock envenenado não impede a troca: o que ele protege é um `Option`
    /// escrito de uma vez, e não uma estrutura que um pânico no meio da escrita
    /// deixaria pela metade.
    #[cfg(feature = "id-snowflake")]
    fn replace(created: SnowflakeIdGenerator) {
        *DATABASE.write().unwrap_or_else(PoisonError::into_inner) = Some(created);
    }

    /// O gerador guardado, criando o da instância zero se ainda não houver um.
    ///
    /// A leitura vem primeiro porque é o caminho de sempre: só a primeira
    /// chamada de um processo que não passou pelo boot pede o lock de escrita.
    #[cfg(feature = "id-snowflake")]
    fn shared() -> SnowflakeIdGenerator {
        let installed = DATABASE
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();

        if let Some(existing) = installed {
            return existing;
        }

        let secrets = DomainSecrets::default();

        DATABASE
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .get_or_insert_with(|| snowflake_id_generator(secrets.cluster_id, secrets.server_id))
            .clone()
    }
}
