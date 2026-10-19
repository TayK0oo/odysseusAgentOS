//! Tests adversariaux — `omega-search` (enhance_query / rank_results).
//!
//! Le traitement de requête touche des entrées non fiables (texte utilisateur)
//! : on vérifie l'absence de panique, de déni de service et de fuite de scope.

use omega_search::{
    cache_duration_secs, detect_question_type, domain, enhance_query, extract_entities,
    rank_results, split_multi_part, SearchHit,
};

#[test]
fn atk_query_never_panics() {
    let hostile = [
        "",
        " ",
        "\n\n",
        "\0",
        "site:",
        "site: ",
        ":::",
        "and",
        "and and and",
        "or or or",
        "; ; ;",
        "what",
        "how to",
        "site:example.com site:evil.com",
        &"A".repeat(100_000),
        &"🇫🇷🎉<>[]{}\"'; DROP TABLE".repeat(1_000),
        "who what when where why how",
    ];
    for q in hostile {
        let _ = detect_question_type(q);
        let _ = extract_entities(q);
        let _ = split_multi_part(q);
        let _ = enhance_query(q);
        let _ = cache_duration_secs(q);
    }
}

#[test]
fn atk_site_filter_cannot_inject_query() {
    // Un token site: vide ne doit pas produire une requête vide piégée.
    let e = enhance_query("site:");
    assert!(e.site.is_none() || e.site.as_deref().map(|s| !s.is_empty()).unwrap_or(true));
    // Deux filtres site: -> on prend le premier, le reste part dans la requête.
    let e2 = enhance_query("site:a.com site:b.com");
    assert!(
        !e2.query.contains("site:"),
        "un filtre site: a survécu : {}",
        e2.query
    );
}

#[test]
fn atk_search_never_injects_shell_metachars() {
    // Les métacaractères shell traversent enhance_query sans être exécutés
    // (le port ne shell-out jamais) — on verrouille l'absence de ré-écriture.
    let evil = "; rm -rf / && echo $(whoami)";
    let e = enhance_query(evil);
    assert!(
        e.query.contains("rm -rf /"),
        "la requête doit rester intacte (pas de réécriture)"
    );
}

#[test]
fn atk_glob_pattern_in_query_does_not_recurse() {
    // Un motif glob/pathResolver n'est pas interprété par le ranker.
    let hits = vec![SearchHit {
        title: "**".into(),
        url: "file:///etc/passwd".into(),
        snippet: "../../../etc/passwd".into(),
        age: None,
    }];
    let ranked = rank_results("** ../../", hits);
    assert_eq!(ranked.len(), 1);
    assert!(
        ranked[0].snippet.chars().count() <= 200,
        "snippet non borné"
    );
}

#[test]
fn atk_domain_extraction_is_total() {
    for u in [
        "",
        "http://",
        "https://",
        "://",
        "not a url",
        "ftp://a.b.c/d",
        "javascript:alert(1)",
    ] {
        let _ = domain(u); // ne doit pas paniquer
    }
    assert_eq!(domain("file:///etc/passwd"), None);
    assert_eq!(domain("javascript:alert(1)"), None);
}

#[test]
fn atk_rank_bounds_output() {
    // 10 000 résultats -> la fonction doit rester rapide et ordonnée.
    let hits: Vec<SearchHit> = (0..10_000)
        .map(|i| SearchHit {
            title: format!("result {}", i),
            url: format!("https://site{}.com/{}", i % 50, i),
            snippet: "x".repeat(5_000),
            age: Some("1 hour ago".into()),
        })
        .collect();
    let ranked = rank_results("result", hits);
    assert_eq!(ranked.len(), 10_000);
    assert!(ranked.iter().all(|h| h.snippet.chars().count() <= 200));
}

#[test]
fn atk_cache_duration_is_bounded() {
    // news → 30 min, sinon 24 h : jamais 0 ni inf.
    for q in ["news", "actualités", "breaking", "", "x"] {
        let d = cache_duration_secs(q);
        assert!(
            d > 0 && d <= 86_400,
            "durée hors bornes pour {:?}: {}",
            q,
            d
        );
    }
}
