//! Tests v24 — `omega-search`.

#[test]
fn detects_question_type_whole_word() {
    assert_eq!(
        omega_search::detect_question_type("what is rust"),
        Some("what")
    );
    assert_eq!(omega_search::detect_question_type("whatsapp pricing"), None);
}

#[test]
fn enhance_extracts_site_filter() {
    let e = omega_search::enhance_query("best tools site:github.com");
    assert_eq!(e.site.as_deref(), Some("github.com"));
    assert!(!e.query.contains("site:"));
}

#[test]
fn enhance_multi_part() {
    let single = omega_search::enhance_query("rust ownership");
    assert!(!single.multi_part);
    let multi = omega_search::enhance_query("rust ownership and borrow checker");
    assert!(multi.multi_part);
}

#[test]
fn cache_duration_news() {
    assert_eq!(
        omega_search::cache_duration_secs("latest news about rust"),
        1800
    );
    assert_eq!(
        omega_search::cache_duration_secs("rust documentation"),
        86400
    );
}

#[test]
fn domain_extraction() {
    assert_eq!(
        omega_search::domain("https://docs.rust-lang.org/book").as_deref(),
        Some("rust-lang.org")
    );
    assert!(omega_search::domain("not a url").is_none());
}

#[test]
fn recency_ordering() {
    assert!(
        omega_search::recency_score(Some("1 hour ago"))
            > omega_search::recency_score(Some("3 days ago"))
    );
    assert_eq!(omega_search::recency_score(None), 0.5);
}

#[test]
fn rank_matching_first() {
    let hits = vec![
        omega_search::SearchHit {
            title: "Unrelated cooking recipe".into(),
            url: "https://foo.com/a".into(),
            snippet: "pasta".into(),
            age: Some("2 days ago".into()),
        },
        omega_search::SearchHit {
            title: "Rust ownership rules explained".into(),
            url: "https://doc.rust-lang.org/1".into(),
            snippet: "rust ownership borrow checker rules".into(),
            age: Some("1 hour ago".into()),
        },
    ];
    let ranked = omega_search::rank_results("rust ownership", hits);
    assert_eq!(ranked[0].title, "Rust ownership rules explained");
}

#[test]
fn split_multi_part() {
    let parts = omega_search::split_multi_part("rust and python or go");
    assert_eq!(parts.len(), 3);
}

#[test]
fn is_news_query() {
    assert!(omega_search::is_news_query("latest news about rust"));
    assert!(!omega_search::is_news_query("rust documentation"));
}

#[test]
fn news_quality_in_range() {
    let hit = omega_search::SearchHit {
        title: "Rust ownership rules explained".into(),
        url: "https://doc.rust-lang.org/1".into(),
        snippet: "rust ownership borrow checker rules here".into(),
        age: Some("1 hour ago".into()),
    };
    let q = omega_search::news_quality(&hit);
    assert!(q >= 0.0 && q <= 1.0);
}

#[test]
fn extract_entities() {
    let e = omega_search::extract_entities("Rust release 2026 by Anthropic");
    assert!(e.names.iter().any(|n| n == "Rust" || n == "Anthropic"));
    assert!(e.dates.contains(&"2026".to_string()));
}
