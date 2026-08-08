//! O tamanho de página padrão.

/// Itens por página quando o cliente não pede um limite.
///
/// Vinte é o que o PHP usava e o que a suíte de integração afirma. Não é
/// configurável de propósito: um limite que varia por deploy faria a mesma
/// requisição devolver contagens diferentes em ambientes diferentes.
pub const DEFAULT_LIMIT: u32 = 20;
