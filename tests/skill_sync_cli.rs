use std::fs;
use std::path::Path;

use assert_cmd::Command;

#[test]
fn install_skills_copies_bundled_skills_to_default_and_extra_project_targets() {
    let project = tempfile::tempdir().unwrap();
    let source = tempfile::tempdir().unwrap();
    write_skill(source.path(), "alpha");
    write_skill(source.path(), "beta");

    Command::cargo_bin("skills-janitor")
        .unwrap()
        .current_dir(project.path())
        .arg("install-skills")
        .arg("--source")
        .arg(source.path())
        .args(["--target", "kiro"])
        .assert()
        .success();

    for target in [".claude", ".agents", ".kiro"] {
        for skill in ["alpha", "beta"] {
            let skill_file = project
                .path()
                .join(target)
                .join("skills")
                .join(skill)
                .join("SKILL.md");
            assert!(
                skill_file.is_file(),
                "expected {} to be installed",
                skill_file.display()
            );
        }
    }
}

fn write_skill(root: &Path, name: &str) {
    let skill_dir = root.join(name);
    fs::create_dir_all(skill_dir.join("examples")).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {name}\ndescription: Test skill for install verification.\n---\n\n# {name}\n"
        ),
    )
    .unwrap();
    fs::write(skill_dir.join("examples").join("sample.txt"), "sample").unwrap();
}
