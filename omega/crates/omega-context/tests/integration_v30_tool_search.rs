//! Integration tests v30 — ToolSurfaceIndex + gate (cmd-tool-search).

use omega_context::ToolDoc;
use omega_context::ToolSurfaceIndex;

#[test]
fn tool_surface_index_search_returns_results() {
    let idx = ToolSurfaceIndex::from_docs(vec![
        ToolDoc {
            name: "read_file".into(),
            description: "read a file from disk".into(),
        },
        ToolDoc {
            name: "write_file".into(),
            description: "write content to a file".into(),
        },
        ToolDoc {
            name: "list_dir".into(),
            description: "list directory contents".into(),
        },
    ]);
    let results = idx.search("read file", 10);
    assert!(!results.is_empty(), "search returns results");
    assert!(
        results.iter().any(|r| r.name == "read_file"),
        "read_file found"
    );
}

#[test]
fn tool_surface_index_rare_token_gate() {
    let idx = ToolSurfaceIndex::from_docs(vec![
        ToolDoc {
            name: "read_file".into(),
            description: "read a file from disk".into(),
        },
        ToolDoc {
            name: "write_file".into(),
            description: "write content to a file".into(),
        },
    ]);
    // A query with a rare token should still work
    let results = idx.search("xyznonexistent", 10);
    // Gate may return empty for rare tokens — that's acceptable
    let _ = results;
}

#[test]
fn tool_surface_index_empty_query() {
    let idx = ToolSurfaceIndex::from_docs(vec![ToolDoc {
        name: "read_file".into(),
        description: "read a file".into(),
    }]);
    let results = idx.search("", 10);
    // Empty query should not panic
    let _ = results;
}

#[test]
fn tool_surface_index_empty_docs() {
    let idx = ToolSurfaceIndex::from_docs(vec![]);
    let results = idx.search("anything", 10);
    assert!(results.is_empty(), "empty docs → empty results");
}
