//! O apoio dos testes do crate, e os testes que não pertencem a fonte nenhum.
//!
//! ## Por que os mocks nascem aqui, e não na `infra`
//!
//! As ports que os services consomem são traits da `infra` e do `domain`. Pô-las
//! sob `#[automock]` exigiria que a `infra` mencionasse `mockall` no código de
//! produção, atrás de uma feature — os mocks acompanhariam a trait e nunca
//! divergiriam, ao custo de a camada de produção conhecer a ferramenta de teste.
//!
//! A escolha foi a outra: o `mock!` mora aqui e **restata** a assinatura. O
//! custo é real e vale dizer qual é — mudar um método da trait sem mudar o mock
//! não quebra nada até alguém reparar, porque o mock deixa de implementar a
//! trait e é o teste que o usa que para de compilar.

pub(crate) mod factories;
pub(crate) mod mocks;

mod lib_test;
