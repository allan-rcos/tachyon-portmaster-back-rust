//! Os geradores de id que **não** são identidade de entidade.
//!
//! O id de uma entidade nasce no `domain`, porque escolher quem ela é faz parte
//! da regra. Os dois daqui não têm nada a ver com regra de negócio: um é um
//! segredo, o outro é um número de protocolo. Por isso vivem na `infra` e são
//! **exportados** — o `app` os reexporta, e a apresentação os consome.

/// Gera um id opaco e imprevisível.
///
/// O refresh token é o caso: ele precisa ser impossível de adivinhar, que é o
/// oposto do requisito de um id de entidade — este último ordena por tempo
/// justamente para ser previsível ao índice do banco.
/// Os dois pedem `Clone + Send + Sync + 'static` pelo mesmo motivo que
/// [`LoggerFactory`](crate::logging::LoggerFactory): quem os consome é um
/// `tower::Layer`, e o axum exige que um layer seja clonável e compartilhável
/// entre tarefas. Exigir aqui, e não no ponto de uso, é o que evita a
/// apresentação descobrir a restrição como um erro de trait a três camadas de
/// distância — nenhum gerador tem estado, então nenhum paga por isso.
pub trait RandomIdGenerator: Clone + Send + Sync + 'static {
    /// Um id aleatório novo.
    fn next(&self) -> String;
}

/// Gera um id ordenável em string.
///
/// O `request_id` do logger é o caso: precisa ordenar por tempo para que os logs
/// de uma requisição se sequenciem, mas nunca vira chave primária.
pub trait SortableIdGenerator: Clone + Send + Sync + 'static {
    /// Um id novo, ordenável lexicograficamente.
    fn next(&self) -> String;
}

/// Comprimento do refresh token.
///
/// 21 caracteres do alfabeto URL-safe dão cerca de 126 bits de entropia — longe
/// de qualquer força bruta viável, e ainda cabendo num cookie sem incomodar.
const RANDOM_ID_SIZE: usize = 21;

/// Gerador de refresh token, sobre NanoID.
#[derive(Clone, Copy)]
pub(crate) struct NanoIdGenerator;

impl NanoIdGenerator {
    /// Monta o gerador.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl RandomIdGenerator for NanoIdGenerator {
    fn next(&self) -> String {
        nanoid::nanoid!(RANDOM_ID_SIZE)
    }
}

/// Gerador de `request_id`, sobre xid.
#[derive(Clone, Copy)]
pub(crate) struct XidGenerator;

impl XidGenerator {
    /// Monta o gerador.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl SortableIdGenerator for XidGenerator {
    fn next(&self) -> String {
        xid::new().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn o_token_aleatorio_nao_se_repete() {
        let generator = NanoIdGenerator::new();
        let ids: HashSet<String> = (0..1_000).map(|_| generator.next()).collect();

        assert_eq!(ids.len(), 1_000, "houve colisão em 1000 tokens");
    }

    #[test]
    fn o_token_aleatorio_tem_a_entropia_esperada() {
        let generator = NanoIdGenerator::new();
        assert_eq!(generator.next().chars().count(), RANDOM_ID_SIZE);
    }

    #[test]
    fn o_request_id_ordena_pela_emissao() {
        // É a propriedade que faz os logs se sequenciarem sozinhos quando
        // ordenados por esse campo.
        let generator = XidGenerator::new();
        let mut previous = generator.next();

        for _ in 0..100 {
            let current = generator.next();
            assert!(
                current > previous,
                "{current} deveria vir depois de {previous}"
            );
            previous = current;
        }
    }
}
