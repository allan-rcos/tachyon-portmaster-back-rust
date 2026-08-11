//! O leitor tipado do ambiente, e a lista do que estava errado.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

/// Lê variáveis de ambiente e acumula o que não deu para ler.
///
/// Nenhum método falha na hora. Um valor ilegível vira uma linha na lista e a
/// leitura segue com o padrão — é o que faz um boot recusado nomear **todas** as
/// variáveis erradas de uma vez, em vez de obrigar quem faz deploy a corrigir
/// uma, subir de novo, e descobrir a seguinte.
///
/// O ambiente é lido **uma vez**, em [`Self::of_process`], e o que os elos
/// consultam é a cópia. O que se ganha não é desempenho: é que a leitura deixa
/// de depender de um estado global do processo, e um teste monta a fonte com
/// [`Self::of_pairs`] em vez de escrever em `std::env` e serializar-se contra os
/// outros testes.
///
/// É o `EnvSource` do PHP, com uma diferença: lá o elo do JWT lançava exceção no
/// segredo curto, saindo da chain no meio. Aqui ele também acumula, por
/// [`Self::refuse`]. Um segredo fraco continua derrubando o boot — só derruba
/// depois de os outros elos terem contado o que também está errado, que é a
/// única coisa que a exceção precoce impedia.
pub(crate) struct EnvSource {
    /// O ambiente como estava quando o processo subiu.
    vars: HashMap<String, String>,
    /// O que não deu para ler, na ordem em que apareceu.
    errors: Vec<String>,
}

impl EnvSource {
    /// O leitor sobre o ambiente deste processo.
    pub(crate) fn of_process() -> Self {
        Self::of_pairs(std::env::vars())
    }

    /// O leitor sobre um ambiente dado.
    ///
    /// Variável presente e vazia é descartada aqui, na entrada: um compose que
    /// declara a chave sem valor não deve derrotar o padrão, porque a intenção
    /// de quem escreveu `APP_HOST:` é não escolher host nenhum.
    pub(crate) fn of_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            vars: pairs
                .into_iter()
                .filter(|(_, value)| !value.trim().is_empty())
                .collect(),
            errors: Vec::new(),
        }
    }

    /// Uma variável de texto, com padrão.
    pub(crate) fn string(&self, name: &str, default: &str) -> String {
        self.raw(name).unwrap_or(default).to_owned()
    }

    /// Uma variável de texto obrigatória.
    ///
    /// Ausente, registra a queixa e devolve string vazia. O vazio nunca chega a
    /// ser usado: quem lê a lista de erros recusa o boot antes de o draft virar
    /// configuração.
    pub(crate) fn required(&mut self, name: &str) -> String {
        let Some(value) = self.raw(name).map(str::to_owned) else {
            self.refuse(format!(
                "a variável {name} é obrigatória e não está definida"
            ));

            return String::new();
        };

        value
    }

    /// Uma variável numérica, com padrão.
    ///
    /// Valor presente mas ilegível é **queixa**, não queda no padrão: quem
    /// escreveu `APP_PORT=oito mil` quis dizer alguma coisa, e subir na 8000
    /// esconderia o engano até alguém notar que o serviço está na porta errada.
    pub(crate) fn number<T: FromStr>(&mut self, name: &str, default: T) -> T
    where
        T::Err: std::fmt::Display,
    {
        let Some(raw) = self.raw(name).map(str::to_owned) else {
            return default;
        };

        match raw.parse() {
            Ok(value) => value,
            Err(error) => {
                self.refuse(format!("{name} não é um número válido: {raw} ({error})"));

                default
            }
        }
    }

    /// Uma variável de segundos, com padrão.
    pub(crate) fn duration(&mut self, name: &str, default: Duration) -> Duration {
        Duration::from_secs(self.number(name, default.as_secs()))
    }

    /// Uma variável booleana, com padrão.
    ///
    /// Grafia irreconhecível conta como falso em vez de virar queixa: as
    /// afirmativas cobrem tudo que um compose ou um shell produzem, e qualquer
    /// outra coisa é a intenção de desligar.
    pub(crate) fn flag(&self, name: &str, default: bool) -> bool {
        match self.raw(name) {
            None => default,
            Some(raw) => matches!(
                raw.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            ),
        }
    }

    /// Uma variável de lista separada por vírgula, com padrão.
    pub(crate) fn list(&self, name: &str, default: &[String]) -> Vec<String> {
        let Some(raw) = self.raw(name) else {
            return default.to_vec();
        };

        raw.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Registra uma queixa que o elo descobriu sozinho.
    ///
    /// É o que um elo usa quando o problema não é a leitura e sim o valor — um
    /// segredo curto demais, um modo de TLS que não existe. A leitura continua,
    /// e o boot é recusado no fim com esta linha junto das outras.
    pub(crate) fn refuse(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    /// O veredito, depois de a chain inteira ter rodado.
    pub(crate) fn into_result(self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.errors.is_empty(),
            "variáveis de ambiente inválidas:\n  - {}",
            self.errors.join("\n  - ")
        );

        Ok(())
    }

    /// O valor cru de uma variável.
    fn raw(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Uma fonte sobre um punhado de variáveis.
    fn source(pairs: &[(&str, &str)]) -> EnvSource {
        EnvSource::of_pairs(
            pairs
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())),
        )
    }

    #[test]
    fn variavel_vazia_conta_como_ausente() {
        assert_eq!(source(&[("APP_HOST", "   ")]).string("APP_HOST", "padrão"), "padrão");
    }

    #[test]
    fn numero_ilegivel_e_queixa_e_nao_padrao() {
        let mut env = source(&[("APP_PORT", "oito mil")]);

        assert_eq!(env.number("APP_PORT", 8000_u16), 8000);
        assert!(env.into_result().is_err());
    }

    #[test]
    fn o_booleano_so_e_verdadeiro_nas_grafias_afirmativas() {
        for afirmativo in ["1", "true", "TRUE", "yes", "on"] {
            assert!(source(&[("F", afirmativo)]).flag("F", false), "{afirmativo}");
        }

        for negativo in ["0", "false", "no", "qualquer coisa"] {
            assert!(!source(&[("F", negativo)]).flag("F", true), "{negativo}");
        }
    }

    #[test]
    fn a_lista_ignora_espaco_e_entrada_vazia() {
        assert_eq!(
            source(&[("O", "https://a.test, ,https://b.test ")]).list("O", &[]),
            ["https://a.test".to_owned(), "https://b.test".to_owned()]
        );
    }

    /// É o ponto do desenho: um boot recusado conta tudo de uma vez.
    #[test]
    fn as_queixas_se_acumulam_em_vez_de_parar_na_primeira() {
        let mut env = source(&[]);
        env.required("PRIMEIRA");
        env.required("SEGUNDA");

        let message = env
            .into_result()
            .expect_err("duas ausências recusam o boot")
            .to_string();

        assert!(message.contains("PRIMEIRA"), "{message}");
        assert!(message.contains("SEGUNDA"), "{message}");
    }
}
