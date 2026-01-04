use barforge_types::{ModuleCategory, RegistryModule};

pub fn sample_module(uuid: &str, name: &str) -> RegistryModule {
    RegistryModule {
        uuid: uuid.to_string(),
        name: name.to_string(),
        description: "Example module".to_string(),
        author: "barforge".to_string(),
        category: ModuleCategory::Weather,
        icon: None,
        screenshot: None,
        repo_url: "https://github.com/barforge/example".to_string(),
        downloads: 0,
        version: None,
        last_updated: None,
        rating: None,
        verified_author: true,
        tags: vec!["example".to_string()],
        checksum: None,
        license: None,
    }
}
