use barforge_desktop::state::InstalledModule;
use barforge_desktop::ui::UpdatesShell;
use dioxus::prelude::*;
use semver::Version;

fn version(value: &str) -> Version {
    Version::parse(value).expect("version should parse")
}

#[test]
fn updates_shell_renders_update_summary_and_rows() {
    let modules = vec![
        InstalledModule {
            name: "weather".to_string(),
            installed_version: version("1.0.0"),
            registry_version: Some(version("1.2.0")),
        },
        InstalledModule {
            name: "clock".to_string(),
            installed_version: version("1.0.0"),
            registry_version: Some(version("1.0.0")),
        },
        InstalledModule {
            name: "cpu".to_string(),
            installed_version: version("0.9.0"),
            registry_version: Some(version("1.1.0")),
        },
    ];

    let html = dioxus_ssr::render_element(rsx!(UpdatesShell {
        modules: modules.clone(),
        updating_all: false,
    }));

    assert!(html.contains("Updates"));
    assert!(html.contains("2 updates available"));
    assert!(html.contains("Update All (2)"));
    assert!(html.contains("weather"));
    assert!(html.contains("cpu"));
}

#[test]
fn updates_shell_renders_no_updates_state() {
    let modules = vec![InstalledModule {
        name: "clock".to_string(),
        installed_version: version("1.0.0"),
        registry_version: Some(version("1.0.0")),
    }];

    let html = dioxus_ssr::render_element(rsx!(UpdatesShell {
        modules,
        updating_all: false,
    }));

    assert!(html.contains("All modules are up to date"));
    assert!(html.contains("No Updates"));
    assert!(!html.contains("Update All"));
}
