//! Integration tests v30 — SkillIndex + cache (cmd-skill-index).

use omega_context::{Skill, SkillIndex};

#[test]
fn skill_index_from_skills() {
    let skills = vec![
        Skill::new(
            "rust-ownership",
            "rust-ownership",
            "Rust ownership rules",
            "content about ownership",
        ),
        Skill::new(
            "python-basics",
            "python-basics",
            "Python basics",
            "content about python",
        ),
    ];
    let idx = SkillIndex::from_skills(skills);
    assert!(!idx.list().is_empty());
}

#[test]
fn skill_index_empty() {
    let idx = SkillIndex::from_skills(vec![]);
    assert!(idx.list().is_empty());
}

#[test]
fn skill_index_view() {
    let skills = vec![Skill::new(
        "rust-ownership",
        "rust-ownership",
        "Rust ownership rules",
        "content about ownership",
    )];
    let idx = SkillIndex::from_skills(skills);
    assert!(idx.view("rust-ownership").is_some());
    assert!(idx.view("nonexistent").is_none());
}
