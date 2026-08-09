//! O contrato do relógio.

use chrono::{DateTime, Utc};

/// De onde o sistema tira a hora corrente.
///
/// Existe para que ninguém chame `Instant::now` nem `SystemTime::now` solto —
/// os dois estão proibidos no `.clippy.toml`, e a proibição não é
/// preciosismo: uma hora tirada do ar não dá para fixar num teste, e um trecho
/// que depende dela só falha na virada do dia.
///
/// Sempre UTC. O sistema é UTC de ponta a ponta — os modelos carregam
/// `DateTime<Utc>`, a sessão do pool é fixada em `+00:00` — e um segundo fuso
/// em qualquer ponto produziria registros que não ordenam entre si.
///
/// Serve também para medir duração: dois `now()` e uma subtração dizem quanto
/// tempo passou. Não é o relógio monotônico do `Instant`, e para latência de
/// requisição isso não muda nada — um ajuste de NTP no meio de uma resposta de
/// milissegundos é um evento que não acontece, e se acontecesse, um `duration_ms`
/// esquisito numa linha de log é um preço que se paga de bom grado por não ter
/// dois relógios no sistema.
pub trait Clock: Clone + Send + Sync + 'static {
    /// A hora corrente, em UTC.
    fn now(&self) -> DateTime<Utc>;
}
