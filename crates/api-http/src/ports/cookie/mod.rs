//! Os cookies que carregam a sessão.
//!
//! `HttpOnly` em ambos, sempre. É o que impede um XSS de ler o token por
//! JavaScript — a diferença entre um script injetado poder incomodar o usuário
//! e poder roubar a sessão dele inteira.
//!
//! O resto da política — validade, `Secure`, `SameSite` — mora na
//! [`SessionPolicy`](crate::ports::session_policy::SessionPolicy), decidida em
//! compilação. Aqui fica só o que é do cookie em si: sob que nome ele viaja, e
//! quem sabe montá-lo.

pub(crate) mod auth_cookie;
pub(crate) mod cookie_name;

pub(crate) mod adapter;
