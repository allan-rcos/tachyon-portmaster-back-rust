//! Os cookies que carregam a sessão.
//!
//! Só o **vocabulário** mora aqui: sob que nome cada cookie viaja. Quem os monta
//! e os lê é o contexto de cookie do middleware, e é o único ponto do sistema
//! que conhece o tipo `Cookie`.
//!
//! Havia uma trait `AuthCookie` tentando abstrair aquele tipo, e ela não
//! abstraía nada: quatro dos seus seis métodos o devolviam na assinatura, então
//! ele atravessava o contrato e chegava aos controllers de qualquer forma. O que
//! restou dela é a [`CookiePort`](crate::middleware::cookie_port::CookiePort),
//! que fala em nome e valor.

pub(crate) mod cookie_name;
