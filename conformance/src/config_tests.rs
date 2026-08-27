//! Config merge conformance using owned fixtures.

use jerekode_config::{ConfigLayer, ConfigLoader};
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/config")
        .join(path)
}

#[test]
fn loads_minimal_opencode_fixture() {
    let mut loader = ConfigLoader::new();
    loader
        .load_file(fixture("opencode_minimal.json"), ConfigLayer::Global)
        .unwrap();
    assert_eq!(loader.opencode().provider.as_deref(), Some("openai"));
    assert_eq!(loader.opencode().port, Some(4096));
}

#[test]
fn jsonc_fixture_parses_comments() {
    let mut loader = ConfigLoader::new();
    loader
        .load_file(
            fixture("opencode_with_comments.jsonc"),
            ConfigLayer::Project,
        )
        .unwrap();
    assert_eq!(loader.opencode().provider.as_deref(), Some("anthropic"));
}

#[test]
fn project_override_fixture_wins_on_scalars() {
    let mut loader = ConfigLoader::new();
    loader
        .load_file(fixture("opencode_minimal.json"), ConfigLayer::Global)
        .unwrap();
    loader
        .load_file(
            fixture("opencode_project_override.json"),
            ConfigLayer::Project,
        )
        .unwrap();
    assert_eq!(loader.opencode().provider.as_deref(), Some("openai"));
    assert_eq!(loader.opencode().model.as_deref(), Some("gpt-4o-mini"));
    assert_eq!(loader.opencode().port, Some(8080));
}

#[test]
fn tui_minimal_fixture_loads() {
    let mut loader = ConfigLoader::new();
    loader
        .load_file(fixture("tui_minimal.json"), ConfigLayer::Global)
        .unwrap();
    assert_eq!(loader.tui().theme.as_deref(), Some("dark"));
}
