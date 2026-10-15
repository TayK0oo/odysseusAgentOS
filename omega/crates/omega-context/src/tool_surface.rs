//! Surface d'outils : index BM25 simplifie + gate par le token le plus rare.

use std::collections::{HashMap, HashSet};

/// Document outil indexe (nom + description ; le schema est differe, cf. `ToolSpec`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDoc {
    pub name: String,
    pub description: String,
}

impl ToolDoc {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// Tokenise en minuscules puis decoupe sur les caracteres non alphanumeriques.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

struct IndexedDoc {
    doc: ToolDoc,
    tokens: Vec<String>,
}

/// Index de surface d'outils : **BM25 simplifie** (score = somme des IDF des termes de
/// la requete presents dans le document) avec **gate par le token le plus rare**.
///
/// Admission d'un outil pour une requete :
/// - il contient le token de plus haut IDF de la requete (le mot qui nomme l'intention), ou
/// - (`min_score > 0`) son score atteint le seuil `min_score` (le « OU depasse un seuil »).
///
/// Deterministe : aucun LLM, tri stable (score decroissant, nom croissant).
#[derive(Default)]
pub struct ToolSurfaceIndex {
    docs: Vec<IndexedDoc>,
    df: HashMap<String, usize>,
    idf: HashMap<String, f64>,
    min_score: f64,
}

impl ToolSurfaceIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construit l'index depuis une liste de documents.
    pub fn from_docs(docs: Vec<ToolDoc>) -> Self {
        let mut idx = Self::new();
        for doc in docs {
            idx.add(doc);
        }
        idx
    }

    /// Active le second chemin d'admission : `score >= min_score`. `0.0` (defaut) le
    /// desactive : seule la gate s'applique.
    pub fn with_min_score(mut self, min_score: f64) -> Self {
        self.min_score = min_score;
        self
    }

    /// Ajoute un outil et reconstruit les statistiques (df/idf).
    pub fn add(&mut self, doc: ToolDoc) {
        let tokens = tokenize(&format!("{} {}", doc.name, doc.description));
        self.docs.push(IndexedDoc { doc, tokens });
        self.rebuild_stats();
    }

    fn rebuild_stats(&mut self) {
        let n = self.docs.len();
        self.df.clear();
        for d in &self.docs {
            let mut seen: HashSet<&str> = HashSet::new();
            for t in &d.tokens {
                if seen.insert(t.as_str()) {
                    *self.df.entry(t.clone()).or_insert(0) += 1;
                }
            }
        }
        self.idf.clear();
        for (t, &df) in &self.df {
            // IDF lisse BM25 : ln((N+1)/(df+1)) + 1.
            let idf = (((n + 1) as f64) / ((df + 1) as f64)).ln() + 1.0;
            self.idf.insert(t.clone(), idf);
        }
    }

    /// Token de plus haut IDF de la requete, parmi ceux presents dans le corpus.
    /// Egalite -> premier dans l'ordre de la requete (determinisme).
    fn gate_token(&self, query_tokens: &[String]) -> Option<String> {
        let mut best: Option<(f64, usize, String)> = None;
        for (i, t) in query_tokens.iter().enumerate() {
            if let Some(&idf) = self.idf.get(t) {
                let replace = match &best {
                    None => true,
                    Some((best_idf, best_i, _)) => {
                        idf > *best_idf || (idf == *best_idf && i < *best_i)
                    }
                };
                if replace {
                    best = Some((idf, i, t.clone()));
                }
            }
        }
        best.map(|(_, _, t)| t)
    }

    /// Recherche les `top_k` outils pertinents pour une requete.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<&ToolDoc> {
        let query_tokens = tokenize(query);
        let Some(gate) = self.gate_token(&query_tokens) else {
            return Vec::new();
        };

        let mut scored: Vec<(f64, &ToolDoc)> = Vec::new();
        for d in &self.docs {
            let contains_gate = d.tokens.contains(&gate);
            let mut score = 0.0;
            for t in &query_tokens {
                if d.tokens.contains(t) {
                    if let Some(idf) = self.idf.get(t) {
                        score += idf;
                    }
                }
            }
            let admitted = if self.min_score > 0.0 {
                contains_gate || score >= self.min_score
            } else {
                contains_gate
            };
            if admitted {
                scored.push((score, &d.doc));
            }
        }

        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        scored.into_iter().take(top_k).map(|(_, doc)| doc).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(name: &str, desc: &str) -> ToolDoc {
        ToolDoc::new(name, desc)
    }

    #[test]
    fn rarest_token_gate_excludes_common_only_tools() {
        // Cas Hermes : "common alpha" ne renvoie PAS les outils qui ne partagent que le
        // token commun "common".
        let idx = ToolSurfaceIndex::from_docs(vec![
            tool("alpha_tool", "common common alpha"),
            tool("beta_tool", "common common beta"),
            tool("gamma_tool", "common common gamma"),
            tool("delta_tool", "common common delta"),
        ]);
        let names: Vec<&str> = idx
            .search("common alpha", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha_tool"]);
    }

    #[test]
    fn threshold_admits_strong_match_without_gate_token() {
        let docs = vec![tool("a", "x y"), tool("b", "x"), tool("c", "z")];

        // Sans seuil : seule la gate (token le plus rare "y") admet -> "a" seulement.
        let strict = ToolSurfaceIndex::from_docs(docs.clone());
        let names: Vec<&str> = strict
            .search("x y", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["a"]);

        // Avec seuil : "b" (score = idf(x) ~1.29) atteint le seuil 0.5 -> admis.
        let loose = ToolSurfaceIndex::from_docs(docs).with_min_score(0.5);
        let names: Vec<&str> = loose
            .search("x y", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn results_are_sorted_by_score_then_name_and_limited() {
        let idx = ToolSurfaceIndex::from_docs(vec![
            tool("middle", "rare common"),
            tool("top", "rare common common"),
            tool("bottom", "rare"),
        ])
        .with_min_score(1.0);

        // gate = "common" (plus rare). "top"/"middle" l'ont ; "bottom" passe par le seuil.
        let names: Vec<&str> = idx
            .search("rare common", 3)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["middle", "top", "bottom"]);

        // top_k borne le resultat.
        let top2: Vec<&str> = idx
            .search("rare common", 2)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(top2, vec!["middle", "top"]);
    }

    #[test]
    fn unknown_query_returns_empty() {
        let idx = ToolSurfaceIndex::from_docs(vec![tool("a", "alpha")]);
        assert!(idx.search("inconnu", 10).is_empty());
    }
}
