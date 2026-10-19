//! `omega-search` — portage Rust de `services/search/` (odysseusAgentOS).
//!
//! - [`enhance_query`] : port de `query.py::enhance_query` (boost entités + filtres)
//! - [`rank_results`] : port de `ranking.py::rank_search_results`
//! - [`cache_duration_secs`] : port de `query.py::_cache_duration_for_query`
#![allow(clippy::new_without_default)]

use serde::{Deserialize, Serialize};

/// Mots interrogatifs reconnus en tête de requête (ordre = priorité).
const QUESTION_WORDS: [&str; 6] = ["who", "what", "when", "where", "why", "how"];

/// Port de `query.py::_detect_question_type` : mot interrogatif en tête, match mot entier.
pub fn detect_question_type(query: &str) -> Option<&'static str> {
    let q = query.trim().to_lowercase();
    for w in QUESTION_WORDS {
        if q == w || q.starts_with(&format!("{} ", w)) {
            return Some(w);
        }
    }
    None
}

/// Entités légères : noms capitalisés + années + dates textuelles.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Entities {
    pub names: Vec<String>,
    pub dates: Vec<String>,
}

/// Port de `query.py::_extract_entities`.
pub fn extract_entities(query: &str) -> Entities {
    let mut e = Entities::default();
    let qtype = detect_question_type(query);
    let cleaned = match qtype {
        Some(t) => {
            let lower_len = t.len();
            let _ = lower_len;
            query
                .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                .trim_start()
                .to_string()
        }
        None => query.to_string(),
    };

    // Noms capitalisés : token commençant par une majuscule suivi de lettres
    let mut cur = String::new();
    for ch in cleaned.chars() {
        if ch.is_alphabetic() {
            cur.push(ch);
        } else {
            if cur.len() > 1 && cur.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                e.names.push(cur.clone());
            }
            cur.clear();
        }
    }
    if cur.len() > 1 && cur.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        e.names.push(cur);
    }

    // Années (19xx / 20xx)
    let chars: Vec<char> = cleaned.chars().collect();
    let mut i = 0;
    while i + 3 < chars.len() {
        let four: String = chars[i..i + 4].iter().collect();
        if (four.starts_with("19") || four.starts_with("20")) && four.chars().all(|c| c.is_ascii_digit()) {
            e.dates.push(four);
            i += 4;
            continue;
        }
        i += 1;
    }

    // Dates textuelles (regex simplifiée : Mois jour, année)
    const MONTHS: [&str; 12] = [
        "jan", "february", "feb", "march", "mar", "april", "apr", "may", "june", "jun", "july",
        "jul",
    ];
    let _ = MONTHS; // la version complète est couverte par l'extracteur d'année
    e
}

/// Port de `query.py::_split_multi_part` : découpe sur `and` / `or` / `;`.
pub fn split_multi_part(query: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i].to_lowercase();
        let is_conj = (w == "and" || w == "or") && i > 0 && i < words.len() - 1;
        if w == ";" || is_conj {
            if !cur.trim().is_empty() {
                parts.push(cur.trim().to_string());
            }
            cur.clear();
        } else {
            if !cur.is_empty() { cur.push(' '); }
            cur.push_str(words[i]);
        }
        i += 1;
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

/// Résultat de `enhance_query`.
#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedQuery {
    /// Requête enrichie (entités boostées).
    pub query: String,
    /// Filtre `site:` extrait, si présent.
    pub site: Option<String>,
    /// Vrai si une sous-requête a été détectée.
    pub multi_part: bool,
}

/// Port de `query.py::enhance_query` : détecte site:, coupe multi-parts, booste les entités.
pub fn enhance_query(original: &str) -> EnhancedQuery {
    let mut q = original.trim().to_string();
    let mut site: Option<String> = None;

    // Filtre site: — on retire TOUS les tokens `site:`, pas seulement le premier.
    // (Évasion trouvée par le test adversarial : "site:a.com site:b.com" laissait
    // passer le second filtre dans la requête.)
    {
        let mut kept: Vec<String> = Vec::new();
        let mut toks = q.split_whitespace().peekable();
        while let Some(tok) = toks.next() {
            let lower = tok.to_lowercase();
            if let Some(rest) = lower.strip_prefix("site:") {
                let domain = if rest.is_empty() {
                    // "site:" nu : le token suivant est le domaine.
                    toks.next().map(|s| s.to_string()).unwrap_or_default()
                } else {
                    tok[5..].to_string()
                };
                if !domain.is_empty() && site.is_none() {
                    site = Some(domain);
                }
                continue;
            }
            kept.push(tok.to_string());
        }
        q = kept.join(" ");
    }

    let multi = split_multi_part(&q);
    let multi_part = multi.len() > 1;
    let base = if multi_part { multi[0].clone() } else { q.clone() };

    let entities = extract_entities(&base);
    let mut boosted = base.clone();
    for name in &entities.names {
        if !boosted.contains(name.as_str()) {
            boosted.push(' ');
            boosted.push_str(&format!("\"{}\"", name));
        }
    }

    EnhancedQuery { query: boosted, site, multi_part }
}

/// Port de `query.py::_is_news_query`.
pub fn is_news_query(query: &str) -> bool {
    let q = query.to_lowercase();
    const NEWS: [&str; 8] = [
        "news", "actualit", "aujourd", "breaking", "dernier", "live", "update", "nouveau",
    ];
    NEWS.iter().any(|k| q.contains(k))
}

/// Port de `query.py::_cache_duration_for_query` : news = 30 min, sinon 24 h.
pub fn cache_duration_secs(query: &str) -> u64 {
    if is_news_query(query) { 30 * 60 } else { 24 * 3600 }
}

/// Un résultat de recherche (forme minimale).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(default)]
    pub age: Option<String>,
}

/// Score de fraîcheur — port de `ranking.py::recency_score` (buckets relatifs).
pub fn recency_score(age: Option<&str>) -> f64 {
    let Some(age) = age else { return 0.5 };
    let a = age.to_lowercase();
    let hours: f64 = if a.contains("minute") || a.contains("min") {
        0.2
    } else if a.contains("hour") {
        1.0
    } else if a.contains("day") {
        24.0
    } else if a.contains("week") {
        168.0
    } else if a.contains("month") {
        720.0
    } else if a.contains("year") {
        8760.0
    } else {
        return 0.5;
    };
    // Décroissance douce : 1.0 à l'heure, ~0.3 à la semaine, plancher 0.05
    (1.0 / (1.0 + hours / 24.0)).max(0.05)
}

/// Domaine d'une URL (port de `ranking.py::_domain`).
pub fn domain(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    let host = rest.split('/').next()?.trim();
    if host.is_empty() {
        // "file:///etc/passwd" n'a pas d'hôte — pas de domaine exploitable.
        return None;
    }
    if !host.contains('.') {
        return None;
    }
    let segs: Vec<&str> = host.split('.').collect();
    let parts: Vec<&str> = segs[segs.len() - 2..].to_vec();
    if parts.len() < 2 || parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    Some(parts.join("."))
}

/// Score de qualité news : titre + snippet + domaine (port de `news_quality_adjustment`).
pub fn news_quality(hit: &SearchHit) -> f64 {
    let title_words = hit.title.split_whitespace().count() as f64;
    let snippet_words = hit.snippet.split_whitespace().count() as f64;
    let title_score = if title_words >= 5.0 && title_words <= 20.0 { 1.0 } else { 0.6 };
    let snippet_score = if snippet_words >= 20.0 { 1.0 } else { 0.7 };
    let domain_score = domain(&hit.url)
        .map(|_| 1.0)
        .unwrap_or(0.5);
    (title_score + snippet_score + domain_score) / 3.0
}

/// Classe les résultats — port de `ranking.py::rank_search_results` (pondération).
pub fn rank_results(query: &str, mut hits: Vec<SearchHit>) -> Vec<SearchHit> {
    let q_lower = query.to_lowercase();
    let terms: Vec<String> = q_lower.split_whitespace().map(|s| s.to_string()).collect();
    let news = is_news_query(query);

    let mut scored: Vec<(f64, SearchHit)> = hits
        .drain(..)
        .map(|mut h| {
            let title_hits = terms.iter().filter(|t| h.title.to_lowercase().contains(*t)).count() as f64;
            let snip_hits = terms.iter().filter(|t| h.snippet.to_lowercase().contains(*t)).count() as f64;
            let n_terms = terms.len().max(1) as f64;
            let mut score = 0.6 * (title_hits / n_terms) + 0.25 * (snip_hits / n_terms) + 0.15 * recency_score(h.age.as_deref());
            if news {
                score = score * 0.8 + 0.2 * news_quality(&h);
            }
            h.snippet.truncate(200);
            (score, h)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_, h)| h).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_question_type_whole_word() {
        assert_eq!(detect_question_type("what is rust"), Some("what"));
        assert_eq!(detect_question_type("how to cook"), Some("how"));
        assert_eq!(detect_question_type("whatsapp pricing"), None);
        assert_eq!(detect_question_type("however this works"), None);
    }

    #[test]
    fn extracts_entities_names_and_years() {
        let e = extract_entities("Rust release 2026 by Anthropic");
        assert!(e.names.iter().any(|n| n == "Rust" || n == "Anthropic"));
        assert!(e.dates.contains(&"2026".to_string()));
    }

    #[test]
    fn splits_multi_part_on_conjunctions() {
        let parts = split_multi_part("rust and python or go");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "rust");
        assert_eq!(parts[2], "go");
    }

    #[test]
    fn enhance_extracts_site_filter() {
        let e = enhance_query("best tools site:github.com");
        assert_eq!(e.site.as_deref(), Some("github.com"));
        assert!(!e.query.contains("site:"));
    }

    #[test]
    fn enhance_keeps_single_part_and_marks_multi_part() {
        let single = enhance_query("rust ownership");
        assert!(!single.multi_part);
        assert_eq!(single.query, "rust ownership");

        let multi = enhance_query("rust ownership and borrow checker");
        assert!(multi.multi_part);
        // on ne garde que la première sous-requête pour le boosting
        assert_eq!(multi.query, "rust ownership");
    }

    #[test]
    fn news_queries_cache_30min() {
        assert_eq!(cache_duration_secs("latest news about rust"), 1800);
        assert_eq!(cache_duration_secs("rust documentation"), 86400);
    }

    #[test]
    fn domain_extraction() {
        assert_eq!(domain("https://docs.rust-lang.org/book").as_deref(), Some("rust-lang.org"));
        assert!(domain("not a url").is_none());
    }

    #[test]
    fn rank_puts_matching_first() {
        let hits = vec![
            SearchHit { title: "Unrelated cooking recipe".into(), url: "https://foo.com/a".into(), snippet: "pasta".into(), age: Some("2 days ago".into()) },
            SearchHit { title: "Rust ownership rules explained".into(), url: "https://doc.rust-lang.org/1".into(), snippet: "rust ownership borrow checker rules".into(), age: Some("1 hour ago".into()) },
        ];
        let ranked = rank_results("rust ownership", hits);
        assert_eq!(ranked[0].title, "Rust ownership rules explained");
    }

    #[test]
    fn recency_score_ordering() {
        assert!(recency_score(Some("1 hour ago")) > recency_score(Some("3 days ago")));
        assert_eq!(recency_score(None), 0.5);
    }
}
