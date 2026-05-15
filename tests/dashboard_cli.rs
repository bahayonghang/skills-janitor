use std::fs;

use assert_cmd::Command;

#[test]
fn dashboard_initializes_from_embedded_template_outside_repo() {
    let cwd = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let output = output_dir.path().join("janitor-dashboard.html");

    Command::cargo_bin("skills-janitor")
        .unwrap()
        .current_dir(cwd.path())
        .args(["dashboard", "--output"])
        .arg(&output)
        .assert()
        .success();

    assert!(!cwd.path().join("templates").exists());

    let html = fs::read_to_string(output).unwrap();
    assert!(html.contains(r#"<script type="application/json" id="snapshotData">"#));
    assert!(html.contains(r#""schema_version""#));
    assert!(html.contains(r#"id="languageToggle""#));
    assert!(html.contains(r#"const i18n = {"#));
    assert!(html.contains("Skills Usage Dashboard"));
    assert!(html.contains("技能使用看板"));
    assert!(!html.contains("Token Cost"));
    assert!(!html.contains("Top waste"));
    assert!(!html.contains("token waste"));
    assert!(!html.contains("Budget "));
}
