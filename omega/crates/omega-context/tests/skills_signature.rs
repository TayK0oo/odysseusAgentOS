//! Contrats comportementaux de `SkillIndex` (module `skills.rs`).
//!
//! Cible : les 2 mutants survivants du fichier `skills.rs`
//! (`mutants.out-context-fresh` : `:51:14` opérateur `^=` → `|=` et `&=`).
//!
//! `signature` est une FNV-1a privée, utilisée comme clé de cache dans
//! `view()`. Le pouvoir discriminant ne vient pas de la valeur du hash mais de
//! l'**invalidation du cache** (et du corps renvoyé) quand deux corps
//! produisent le même hash sous l'opérateur muté mais pas sous `^=`.
//!
//! - `^=` → `|=` : collision construite sur les octets `0x24` et `0x25` :
//!   sous `^=`, les hashs diffèrent (re-render, `render_count == 2`) ;
//!   sous `|=`, `0x25 | 0x24 == 0x25` → hashs identiques → cache conservé
//!   (`render_count == 1`, corps stale).
//! - `^=` → `&=` : collision construite sur les octets `0x25` et `0x27` :
//!   sous `^=`, les hashs diffèrent (re-render) ;
//!   sous `&=`, `0x25 & 0x25 == 0x25 & 0x27 == 0x25` → hashs identiques
//!   (cache conservé, corps stale).

use omega_context::{Skill, SkillIndex};

#[test]
fn xor_signature_invalidates_cache_where_or_would_collide() {
    let mut idx = SkillIndex::from_skills(vec![Skill::new("s", "n", "d", "\x24")]);
    // Premier rendu : cache vide -> re-render, render_count = 1.
    assert_eq!(idx.view("s").unwrap().as_ref(), "\x24");
    assert_eq!(idx.render_count(), 1);

    // Remplacement par un corps dont l'octet XOR-diffère mais OR-collide.
    idx.add(Skill::new("s", "n", "d", "\x25"));
    let second = idx.view("s").unwrap();
    // Originel (`^=`) : hashs différents -> re-render, corps à jour.
    assert_eq!(second.as_ref(), "\x25");
    assert_eq!(idx.render_count(), 2);

    // Mutant (`^=` -> `|=`) : hashs identiques -> cache conservé, corps stale
    // "\x24", render_count resterait 1 -> les assertions ci-dessus échouent.
}

#[test]
fn xor_signature_invalidates_cache_where_and_would_collide() {
    let mut idx = SkillIndex::from_skills(vec![Skill::new("s", "n", "d", "\x25")]);
    assert_eq!(idx.view("s").unwrap().as_ref(), "\x25");
    assert_eq!(idx.render_count(), 1);

    idx.add(Skill::new("s", "n", "d", "\x27"));
    let second = idx.view("s").unwrap();
    // Originel (`^=`) : hashs différents -> re-render, corps à jour.
    assert_eq!(second.as_ref(), "\x27");
    assert_eq!(idx.render_count(), 2);

    // Mutant (`^=` -> `&=`) : hashs identiques -> cache conservé, corps stale
    // "\x25", render_count resterait 1 -> les assertions ci-dessus échouent.
}
