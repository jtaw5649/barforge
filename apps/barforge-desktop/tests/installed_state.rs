use barforge_desktop::state::{InstalledModule, InstalledState};
use semver::Version;

fn version(value: &str) -> Version {
    Version::parse(value).expect("version should parse")
}

#[test]
fn installed_module_has_update_when_registry_newer() {
    let module = InstalledModule {
        name: "weather".to_string(),
        installed_version: version("1.0.0"),
        registry_version: Some(version("1.1.0")),
    };

    assert!(module.has_update());
}

#[test]
fn installed_module_no_update_when_same_or_missing() {
    let same = InstalledModule {
        name: "clock".to_string(),
        installed_version: version("1.0.0"),
        registry_version: Some(version("1.0.0")),
    };
    let missing = InstalledModule {
        name: "media".to_string(),
        installed_version: version("1.0.0"),
        registry_version: None,
    };

    assert!(!same.has_update());
    assert!(!missing.has_update());
}

#[test]
fn installed_state_counts_updates() {
    let state = InstalledState {
        modules: vec![
            InstalledModule {
                name: "weather".to_string(),
                installed_version: version("1.0.0"),
                registry_version: Some(version("1.2.0")),
            },
            InstalledModule {
                name: "clock".to_string(),
                installed_version: version("2.0.0"),
                registry_version: Some(version("2.0.0")),
            },
        ],
    };

    assert_eq!(state.update_count(), 1);
    assert_eq!(state.modules_with_updates()[0].name, "weather");
}
