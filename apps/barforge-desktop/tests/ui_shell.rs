use barforge_desktop::ui::BrowseShell;
use barforge_types::{ModuleCategory, RegistryModule};
use dioxus::prelude::*;

fn make_module(name: &str, category: ModuleCategory) -> RegistryModule {
    RegistryModule {
        uuid: format!("{name}-uuid"),
        name: name.to_string(),
        description: format!("Module {name} description"),
        author: "test-author".to_string(),
        category,
        icon: None,
        screenshot: None,
        repo_url: "https://example.test".to_string(),
        downloads: 0,
        version: None,
        last_updated: None,
        rating: None,
        verified_author: false,
        tags: Vec::new(),
        checksum: None,
        license: None,
    }
}

#[test]
fn browse_shell_renders_filtered_list() {
    let modules = vec![
        make_module("Clock", ModuleCategory::Time),
        make_module("Weather", ModuleCategory::Weather),
    ];

    let html = dioxus_ssr::render_element(rsx!(BrowseShell {
        initial_modules: modules,
        initial_search_query: "clock".to_string(),
    }));

    assert!(html.contains("Clock"));
    assert!(!html.contains("Weather"));
}
