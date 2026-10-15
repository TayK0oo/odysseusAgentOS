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

/// Tokenise en minuscules puis decoupe sur les caracteres non alphanumeriques,
/// puis applique le stemming deterministe des deux cotes (docs ET requetes).
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(stem)
        .collect()
}

/// Stemming deterministe maison (pas de Porter complet : YAGNI).
///
/// Regles EXPLICITES, appliquees dans cet ordre (la premiere qui correspond
/// gagne, une seule transformation par appel — pas de cascade) :
/// - R0 : minuscule (defensif ; `tokenize` a deja lowercase en amont, mais
///   `stem` reste correct en usage unitaire : `Write` -> `writ`).
/// - R1 (exceptions exactes, avant les generiques) : `writing -> writ`,
///   `write -> writ` (conflation : sans R1, `writing - ing = writ != write`
///   et la requete "writing" ne rencontrerait jamais le doc "write").
/// - R2 (EN verbal) : `ing -> ""` si `len >= 5` et reste `>= 2`
///   (ex. `playing -> play` ; `sing` bloque par la garde -> `sing`).
/// - R3 (EN verbal) : `ed -> ""`, memes gardes (`len >= 4`, reste `>= 2` ;
///   ex. `watched -> watch` ; `red` bloque -> `red` ; `read` intact car `ad`).
/// - R4 (EN verbal) : `es -> ""` UNIQUEMENT apres sibilante
///   (`sses|zes|xes|ches|shes|oes` ; ex. `boxes -> box`, `passes -> pass`).
///   `files`/`writes` (en `les`/`tes`) ne correspondent PAS et tombent en R5.
/// - R5 (EN/FR pluriel) : `s -> ""`, memes gardes que R3
///   (ex. `files -> file`, `writes -> write`).
/// - R6 (FR) : `eaux -> eau` (ex. `bureaux -> bureau`, `eaux -> eau`).
/// - R7 (FR pluriel) : `x -> ""`, memes gardes
///   (ex. `chevaux -> chevau` ; limite documentee : pas de `al -> ail`,
///   pas de Porter complet).
/// - Sinon : token inchange.
///
/// Limites assumees (deterministes, testees) : `need -> ne`, `travaux ->
/// travau`. Pas de cascade (une regle max par token).
fn stem(token: &str) -> String {
    let t = token.to_lowercase();
    // R1 — exceptions exactes (conflation write/writing).
    if t == "writing" || t == "write" {
        return "writ".to_string();
    }
    // R2 — `ing`.
    if t.len() >= 5 && t.ends_with("ing") {
        let s = &t[..t.len() - 3];
        if s.len() >= 2 {
            return s.to_string();
        }
    }
    // R3 — `ed`.
    if t.len() >= 4 && t.ends_with("ed") {
        let s = &t[..t.len() - 2];
        if s.len() >= 2 {
            return s.to_string();
        }
    }
    // R4 — `es` apres sibilante uniquement (avant R5 : `boxes -> box`,
    // pas `boxe`).
    const ES_SIBILANTS: [&str; 6] = ["sses", "xes", "zes", "ches", "shes", "oes"];
    if t.len() >= 5 {
        for suf in ES_SIBILANTS {
            if t.ends_with(suf) {
                let s = &t[..t.len() - 2];
                if s.len() >= 2 {
                    return s.to_string();
                }
                break;
            }
        }
    }
    // R5 — pluriel en `s` (`files -> file`, `writes -> write`).
    if t.len() >= 4 && t.ends_with('s') {
        let s = &t[..t.len() - 1];
        if s.len() >= 2 {
            return s.to_string();
        }
    }
    // R6 — `eaux -> eau` (avant R7 : le suffixe finit en `x`).
    if t.len() >= 4 && t.ends_with("eaux") {
        let s = &t[..t.len() - 1];
        if s.len() >= 2 {
            return s.to_string();
        }
    }
    // R7 — pluriel en `x`.
    if t.len() >= 4 && t.ends_with('x') {
        let s = &t[..t.len() - 1];
        if s.len() >= 2 {
            return s.to_string();
        }
    }
    t
}

struct IndexedDoc {
    doc: ToolDoc,
    tokens: Vec<String>,
}

/// Index de surface d'outils : **BM25 simplifie** (score = somme des IDF des termes de
/// la requete presents dans le document) avec **gate par le token le plus rare**.
///
/// Admission d'un outil pour une requete :
/// - il contient le token le plus rare de la requete (plus haut IDF == df minimal ; le mot
///   qui nomme l'intention), ou
/// - (`min_score > 0`) son score atteint le seuil `min_score` (le « OU depasse un seuil »).
///
/// Departage du token de gate : ordre lexical croissant (deterministe, independant de la
/// position des tokens dans la requete).
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

    /// Token le plus discriminant de la requete : celui de **plus haut IDF**, c'est-a-dire
    /// de **frequence documentaire minimale** (`df` minimal — IDF est strictement
    /// decroissante en `df`).
    ///
    /// Departage (egalite de `df`, donc d'IDF) : **ordre lexical croissant**. Cette regle
    /// est deterministe et **independante de la position** des tokens dans la requete
    /// (l'ancien « premier token de la requete » etait fragile : un mot vide ou un
    /// reordonnancement changeait la gate).
    fn gate_token(&self, query_tokens: &[String]) -> Option<String> {
        let mut best: Option<(usize, String)> = None; // (df, token)
        for t in query_tokens {
            let Some(&df) = self.df.get(t) else { continue };
            let replace = match &best {
                None => true,
                Some((best_df, best_tok)) => df < *best_df || (df == *best_df && t < best_tok),
            };
            if replace {
                best = Some((df, t.clone()));
            }
        }
        best.map(|(_, t)| t)
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

    #[test]
    fn gate_tie_break_is_lexical_and_position_independent() {
        // "rare_a" et "rare_b" ont le meme df (1) donc le meme IDF. Departage lexical :
        // "rare_a" < "rare_b" -> gate = "rare_a", quel que soit l'ordre de la requete.
        // (L'ancien tie-break « premier token » aurait choisi "rare_b" pour la requete
        // dans l'ordre inverse.)
        let idx = ToolSurfaceIndex::from_docs(vec![tool("aaa", "rare_a"), tool("bbb", "rare_b")]);
        let forward: Vec<&str> = idx
            .search("rare_b rare_a", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        let reversed: Vec<&str> = idx
            .search("rare_a rare_b", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(forward, vec!["aaa"]);
        assert_eq!(reversed, vec!["aaa"]);
    }

    #[test]
    fn open_file_for_writing_gate_is_deterministic_phase1() {
        // Non-regression REVIEW-W6 M2, toujours valide en inc2 : en gate stricte,
        // "open" (df=1, unique a openssl) reste le token le plus rare — l'egalite
        // `open`/`writ` (df=1 chacun depuis le stemming) se departage au lexical
        // croissant (`open` < `writ`) — donc seul `openssl` est admis.
        // Le rappel de file_write par stemming passe par le seuil
        // (cf. `stemming_rappelle_file_write_via_seuil`).
        let idx = ToolSurfaceIndex::from_docs(vec![
            tool("file_write", "write data to a file"),
            tool("file_read", "read data from a file"),
            tool("openssl", "open ssl certificate"),
        ]);
        let names: Vec<&str> = idx
            .search("open file for writing", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["openssl"]);
    }

    #[test]
    fn stem_table_cas_explicites() {
        // Table des regles R0-R7 : (entree, stem attendu).
        let cases = [
            // R0 lowercase defensif + R1 exceptions (conflation write/writing).
            ("writing", "writ"),
            ("Writing", "writ"),
            ("WRITE", "writ"),
            ("write", "writ"),
            // R2 ing.
            ("playing", "play"),
            ("sing", "sing"), // garde : reste < 2 -> inchange.
            // R3 ed.
            ("watched", "watch"),
            ("red", "red"),   // garde : len < 4 -> inchange.
            ("read", "read"), // `ad`, pas `ed` -> inchange.
            // R4 es apres sibilante.
            ("boxes", "box"),
            ("passes", "pass"),
            // R5 pluriel en s (files/writes ne prenaient PAS R4).
            ("files", "file"),
            ("writes", "write"),
            // R6 eaux -> eau.
            ("bureaux", "bureau"),
            ("eaux", "eau"),
            // R7 pluriel en x (limite documentee : pas de al -> ail).
            ("chevaux", "chevau"),
            // Inchanges (gardes + absence de suffixe).
            ("open", "open"),
            ("file", "file"),
            ("ssl", "ssl"),
            ("for", "for"),
            ("common", "common"),
            ("rare", "rare"),
            ("a", "a"),
            ("x", "x"),
            ("y", "y"),
        ];
        for (input, expected) in cases {
            assert_eq!(stem(input), expected, "stem({input:?})");
        }
    }

    #[test]
    fn stem_conflates_write_forms() {
        // Sans cette conflation, "writing" (requete) ne rencontrerait jamais
        // "write" (docs) et le rappel echouerait meme avec seuil.
        assert_eq!(stem("writing"), "writ");
        assert_eq!(stem("write"), "writ");
        assert_eq!(stem("writing"), stem("write"));
    }

    #[test]
    fn stem_est_deterministe() {
        // Meme entree -> meme sortie, des deux cotes (docs ET requetes).
        let a = tokenize("Open FILE for Writing!");
        let b = tokenize("Open FILE for Writing!");
        assert_eq!(a, b);
        assert_eq!(a, vec!["open", "file", "for", "writ"]);
        assert_eq!(stem("Writing"), stem("writing"));
    }

    #[test]
    fn stemming_rappelle_file_write_via_seuil() {
        // Dette W6 inc2 resolue (cf. W6-contexte §8). Meme fixture que le test
        // de gate stricte. Sans stemming, file_write ne scorerait que
        // idf(file) ~= 1.29 < 2.0 (writing != write) et serait exclu ; avec
        // stemming, file_write matche `file` + `writ` ~= 2.98 >= 2.0 -> admis
        // par le seuil et classe premier. `openssl` reste admis par la gate
        // (`open`) malgre son score ~= 1.69 < 2.0. `file_read` (~= 1.29, sans
        // gate) reste exclu.
        let mk = || {
            ToolSurfaceIndex::from_docs(vec![
                tool("file_write", "write data to a file"),
                tool("file_read", "read data from a file"),
                tool("openssl", "open ssl certificate"),
            ])
            .with_min_score(2.0)
        };
        let idx = mk();
        let names: Vec<&str> = idx
            .search("open file for writing", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, vec!["file_write", "openssl"]);
        assert!(names.contains(&"file_write"));
        // Determinisme : deux appels -> meme resultat.
        let again: Vec<&str> = idx
            .search("open file for writing", 10)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, again);
    }

    #[test]
    fn empty_query_returns_empty() {
        let idx = ToolSurfaceIndex::from_docs(vec![tool("a", "alpha")]);
        assert!(idx.search("", 10).is_empty());
    }

    #[test]
    fn top_k_zero_returns_empty() {
        let idx = ToolSurfaceIndex::from_docs(vec![tool("a", "alpha beta")]);
        assert!(idx.search("alpha", 0).is_empty());
    }
}
