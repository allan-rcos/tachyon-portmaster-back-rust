//! Os cookies que carregam a sessão.
//!
//! `HttpOnly` em ambos, sempre. É o que impede um XSS de ler o token por
//! JavaScript — a diferença entre um script injetado poder incomodar o usuário
//! e poder roubar a sessão dele inteira.
//!
//! `Secure` é configurável porque o ambiente de desenvolvimento roda em HTTP
//! puro, e um cookie `Secure` simplesmente não seria enviado — a sessão nunca
//! funcionaria localmente. Em produção, liga-se.

pub(crate) mod auth_cookie;

pub(crate) mod intern;
