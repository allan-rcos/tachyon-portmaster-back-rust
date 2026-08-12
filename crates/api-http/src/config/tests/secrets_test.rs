//! Os testes de `secrets`.

use super::*;

/// É o modo de falha clássico do `linkme`, e ele falha **silencioso**: uma
/// chain sem elo nenhum não acusa nada, só devolve um rascunho vazio.
#[test]
fn a_slice_de_elos_chega_populada() {
    assert!(
        !CONFIG_LINKS.is_empty(),
        "o linker manteve as seções dos elos"
    );
}

/// Os quatro grupos do rascunho têm dono, e é a chain que prova.
#[test]
fn a_chain_preenche_todos_os_slots() {
    let mut env = EnvSource::of_pairs([]);
    let mut draft = BootDraft::default();

    for link in CONFIG_LINKS {
        (link.read)(&mut env, &mut draft);
    }

    assert!(
        draft.into_secrets().is_ok(),
        "todo slot do rascunho tem um elo que o preenche"
    );
}
