//! Progressive disclosure des skills (3 niveaux) + cache par signature.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Resume de skill (niveau 1 : nom + description, corps non charge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Skill complet : corps (niveau 2) + assets nommes (niveau 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub assets: HashMap<String, String>,
}

impl Skill {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            body: body.into(),
            assets: HashMap::new(),
        }
    }

    pub fn with_asset(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.assets.insert(name.into(), content.into());
        self
    }
}

/// Signature deterministe FNV-1a (64 bits) : le cache est invalide quand le corps change.
fn signature(body: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in body.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

struct CachedBody {
    signature: u64,
    body: Arc<str>,
}

/// Index de skills : `list` (resumes), `view` (corps), `asset` (fichier lie).
/// Les corps sont mis en cache par signature : un rechargement n'est re-rendu qu'en
/// cas de changement (meme id, corps different).
#[derive(Default)]
pub struct SkillIndex {
    skills: Vec<Skill>,
    by_id: HashMap<String, usize>,
    cache: Mutex<HashMap<String, CachedBody>>,
    renders: AtomicUsize,
}

impl SkillIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_skills(skills: Vec<Skill>) -> Self {
        let mut idx = Self::new();
        for skill in skills {
            idx.add(skill);
        }
        idx
    }

    /// Ajoute ou remplace un skill (le remplacement invalide le cache par signature).
    pub fn add(&mut self, skill: Skill) {
        if let Some(&pos) = self.by_id.get(&skill.id) {
            self.skills[pos] = skill;
        } else {
            self.by_id.insert(skill.id.clone(), self.skills.len());
            self.skills.push(skill);
        }
    }

    /// Niveau 1 : noms + descriptions (aucun corps charge).
    pub fn list(&self) -> Vec<SkillSummary> {
        self.skills
            .iter()
            .map(|s| SkillSummary {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
            })
            .collect()
    }

    /// Niveau 2 : corps du skill (cache par signature).
    pub fn view(&self, id: &str) -> Option<Arc<str>> {
        let pos = *self.by_id.get(id)?;
        let skill = &self.skills[pos];
        let sig = signature(&skill.body);
        if let Some(cached) = self.cache.lock().expect("cache empoisonne").get(id) {
            if cached.signature == sig {
                return Some(cached.body.clone());
            }
        }
        let body: Arc<str> = Arc::from(skill.body.as_str());
        self.renders.fetch_add(1, Ordering::Relaxed);
        self.cache.lock().expect("cache empoisonne").insert(
            id.to_string(),
            CachedBody {
                signature: sig,
                body: body.clone(),
            },
        );
        Some(body)
    }

    /// Niveau 3 : asset nomme d'un skill.
    pub fn asset(&self, id: &str, name: &str) -> Option<Arc<str>> {
        let pos = *self.by_id.get(id)?;
        self.skills[pos]
            .assets
            .get(name)
            .map(|c| Arc::from(c.as_str()))
    }

    /// Nombre de rendus reels de corps (hors cache) — observabilite de test.
    pub fn render_count(&self) -> usize {
        self.renders.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Skill {
        Skill::new(
            "search",
            "Recherche",
            "Cherche des fichiers",
            "body: usage de grep",
        )
        .with_asset("README.md", "contenu readme")
    }

    #[test]
    fn list_returns_summaries_without_body() {
        let idx = SkillIndex::from_skills(vec![sample()]);
        let list = idx.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "search");
        assert_eq!(list[0].name, "Recherche");
        assert_eq!(list[0].description, "Cherche des fichiers");
    }

    #[test]
    fn three_levels_disclosure() {
        let idx = SkillIndex::from_skills(vec![sample()]);
        // Niveau 1 : resume.
        assert_eq!(idx.list()[0].description, "Cherche des fichiers");
        // Niveau 2 : corps.
        assert_eq!(idx.view("search").unwrap().as_ref(), "body: usage de grep");
        // Niveau 3 : asset.
        assert_eq!(
            idx.asset("search", "README.md").unwrap().as_ref(),
            "contenu readme"
        );
    }

    #[test]
    fn view_cache_by_signature_avoids_re_render() {
        let idx = SkillIndex::from_skills(vec![sample()]);
        let first = idx.view("search").unwrap();
        let second = idx.view("search").unwrap();
        assert_eq!(first, second);
        assert_eq!(idx.render_count(), 1);
    }

    #[test]
    fn cache_invalidates_when_body_changes() {
        let mut idx = SkillIndex::from_skills(vec![sample()]);
        let first = idx.view("search").unwrap();
        assert_eq!(first.as_ref(), "body: usage de grep");
        assert_eq!(idx.render_count(), 1);

        // Meme id, corps different -> signature differente -> re-rendu.
        idx.add(Skill::new("search", "Recherche", "desc", "body: nouveau"));
        let second = idx.view("search").unwrap();
        assert_eq!(second.as_ref(), "body: nouveau");
        assert_eq!(idx.render_count(), 2);
    }

    #[test]
    fn unknown_skill_returns_none() {
        let idx = SkillIndex::from_skills(vec![sample()]);
        assert!(idx.view("absent").is_none());
        assert!(idx.asset("search", "absent").is_none());
    }
}
