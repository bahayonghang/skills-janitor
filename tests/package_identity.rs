use std::process::Command;

use serde_json::Value;

#[test]
fn package_metadata_has_single_root_package() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .expect("failed to run cargo metadata");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata emitted invalid JSON");
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .expect("cargo metadata JSON missing packages array");

    assert_eq!(packages.len(), 1, "expected exactly one Cargo package");

    let package_name = packages[0]
        .get("name")
        .and_then(Value::as_str)
        .expect("package missing name");
    let legacy_package_name = ["skills", "janitor"].join("-");
    let removed_package_names = [
        ["skillscope", "cli"].join("-"),
        legacy_package_name.clone(),
        [legacy_package_name, "cli".to_owned()].join("-"),
    ];

    assert_eq!(package_name, "skillscope");
    assert!(
        packages
            .iter()
            .filter_map(|package| package.get("name").and_then(Value::as_str))
            .all(|name| !removed_package_names.iter().any(|removed| removed == name)),
        "removed package names must not remain in package metadata"
    );
}
