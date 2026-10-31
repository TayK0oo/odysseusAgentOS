//! Contrats comportementaux des helpers purs de `omega-memory::skill`.

use omega_memory::skill::{parse_frontmatter, parse_list_lines, slugify, Skill};

#[test]
fn slugify_normalizes_names() {
    assert_eq!(
        slugify("Prospection Commerciale"),
        "prospection-commerciale"
    );
    assert_eq!(slugify("  Hello   World!! "), "hello-world");
    assert_eq!(slugify(""), "skill");
    assert_eq!(slugify("a/b\\c"), "a-b-c");
    assert_eq!(slugify("---"), "skill");
}

#[test]
fn parse_frontmatter_splits_yaml_and_body() {
    let md = "---\nname: test\ndescription: Une skill\n---\n\n## When to Use\nQuand il faut\n";
    let (fm, body) = parse_frontmatter(md);
    assert_eq!(fm.get("name").map(String::as_str), Some("test"));
    assert_eq!(fm.get("description").map(String::as_str), Some("Une skill"));
    assert!(body.contains("When to Use"));
}

#[test]
fn parse_list_lines_extracts_bullets() {
    let out = parse_list_lines("- a\n- b\n1. c\n\n");
    assert_eq!(out, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
}

#[test]
fn skill_from_markdown_roundtrip() {
    let md = "---\nname: prospection\ndescription: Trouver des clients\nversion: 1.0.0\n---\n\n## When to Use\nQuand prospecter\n\n## Procedure\n1. Chercher\n2. Contacter\n";
    let s = Skill::from_markdown(md);
    assert_eq!(s.name, "prospection");
    assert_eq!(s.when_to_use, "Quand prospecter");
    assert_eq!(s.procedure, vec!["Chercher", "Contacter"]);
    let md2 = s.to_markdown();
    let back = Skill::from_markdown(&md2);
    assert_eq!(back.name, "prospection");
    assert_eq!(back.procedure, vec!["Chercher", "Contacter"]);
}

#[test]
fn skill_roundtrip_preserves_verification() {
    let s = omega_memory::skill::Skill {
        name: "t2".into(),
        description: "d".into(),
        verification: vec!["v1".into()],
        ..Default::default()
    };
    let md = s.to_markdown();
    let back = omega_memory::skill::Skill::from_markdown(&md);
    assert_eq!(back.verification, vec!["v1"]);
}

#[test]
fn skill_from_markdown_defaults() {
    let s = Skill::from_markdown("---\n---\n");
    assert_eq!(s.name, "");
    assert_eq!(s.version, "1.0.0");
    assert_eq!(s.category, "general");
    assert_eq!(s.status, "draft");
    assert_eq!(s.confidence, 0.8);
    assert_eq!(s.source, "learned");
}

#[test]
fn to_markdown_emits_tags_when_nonempty() {
    let s = omega_memory::skill::Skill {
        name: "t".into(),
        description: "d".into(),
        tags: vec!["x".into()],
        ..Default::default()
    };
    let md = s.to_markdown();
    assert!(md.contains("tags: [x]"), "tags doivent être émis");
}

#[test]
fn to_markdown_omits_tags_when_empty() {
    let s = omega_memory::skill::Skill {
        name: "t".into(),
        description: "d".into(),
        ..Default::default()
    };
    let md = s.to_markdown();
    assert!(!md.contains("tags:"), "pas de ligne tags si vide");
}

#[test]
fn to_markdown_emits_verification_when_nonempty() {
    let s = omega_memory::skill::Skill {
        name: "t".into(),
        description: "d".into(),
        verification: vec!["v1".into()],
        ..Default::default()
    };
    let md = s.to_markdown();
    assert!(
        md.contains("## Verification"),
        "verification doit être émis"
    );
    assert!(md.contains("v1"), "contenu verification");
}

#[test]
fn to_markdown_emits_body_extra_when_nonempty() {
    let s = omega_memory::skill::Skill {
        name: "t".into(),
        description: "d".into(),
        body_extra: "extra body".into(),
        ..Default::default()
    };
    let md = s.to_markdown();
    assert!(md.contains("extra body"), "body_extra doit être émis");
}

#[test]
fn to_markdown_omits_body_extra_when_empty() {
    let s = omega_memory::skill::Skill {
        name: "t".into(),
        description: "d".into(),
        ..Default::default()
    };
    let md = s.to_markdown();
    // body_extra vide : pas de section parasite
    assert!(!md.contains("extra body"));
}

#[test]
fn file_skill_store_load_usage_and_save_usage_roundtrip() {
    use omega_memory::skill::FileSkillStore;
    let dir = std::env::temp_dir().join("omega-skill-usage-rt-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Créer un SKILL.md minimal.
    let skill_dir = dir.join("skills").join("general").join("myskill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: myskill\n---\n").unwrap();

    let store1 = FileSkillStore::new(&dir);
    store1.record_use("myskill");
    store1.record_use("myskill");
    store1.save_usage();

    // Recharger depuis un nouveau store.
    let store2 = FileSkillStore::new(&dir);
    store2.load_usage();
    let skills = store2.all();
    assert!(!skills.is_empty(), "skill doit être chargé");
    assert_eq!(skills[0].uses, 2, "usage doit être persisté et rechargé");

    // Vérifier que le fichier _usage.json existe et contient le compte.
    let usage_path = dir.join("skills").join("_usage.json");
    assert!(usage_path.exists(), "save_usage doit écrire _usage.json");
    let content = std::fs::read_to_string(&usage_path).unwrap();
    assert!(content.contains("\"uses\": 2"), "compte persisté");

    let _ = std::fs::remove_dir_all(&dir);
}
#[test]
fn file_skill_store_name_is_file_skill_store() {
    use omega_memory::skill::FileSkillStore;
    use omega_ports::SkillStore;
    let dir = std::env::temp_dir().join("omega-skill-name-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = FileSkillStore::new(&dir);
    assert_eq!(store.name(), "file-skill-store");
    let _ = std::fs::remove_dir_all(&dir);
}

// (debug removed)
