use barforge_types::{
    Collection, CollectionModule, CollectionOwner, LandingData, LandingInstallMethod,
    ModuleCategory, PublicStats, RegistryModule, UserProfile, UserRole, VersionHistoryEntry,
};
use manganis::asset;

pub(crate) fn sample_user_profile(username: &str, module_count: usize) -> UserProfile {
    UserProfile {
        id: 1,
        username: username.to_string(),
        display_name: Some("Barforge".to_string()),
        avatar_url: None,
        bio: Some("Building the Barforge ecosystem.".to_string()),
        website_url: Some("https://barforge.dev".to_string()),
        github_url: Some("https://github.com/barforge".to_string()),
        twitter_url: Some("https://x.com/barforge".to_string()),
        bluesky_url: Some("https://bsky.app/profile/barforge.dev".to_string()),
        discord_url: Some("https://discord.gg/barforge".to_string()),
        sponsor_url: None,
        verified_author: true,
        role: UserRole::User,
        module_count: module_count as i64,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

pub(crate) fn sample_landing_data() -> LandingData {
    LandingData {
        stats: PublicStats {
            total_modules: 128,
            total_downloads: 24800,
            total_authors: 42,
        },
        install_methods: vec![
            LandingInstallMethod {
                id: "shell".to_string(),
                label: "Shell".to_string(),
                description: "Install via a one-liner shell script.".to_string(),
                commands: vec!["curl -sSL https://barforge.dev/install | sh".to_string()],
            },
            LandingInstallMethod {
                id: "aur".to_string(),
                label: "AUR".to_string(),
                description: "Install from the Arch User Repository.".to_string(),
                commands: vec!["yay -S barforge".to_string()],
            },
            LandingInstallMethod {
                id: "source".to_string(),
                label: "Source".to_string(),
                description: "Build from source with Cargo.".to_string(),
                commands: vec![
                    concat!(
                        "git clone https://github.com/jtaw5649/barforge-app && ",
                        "cd barforge-app && cargo build --release"
                    )
                    .to_string(),
                ],
            },
        ],
    }
}

pub(crate) fn sample_collections(_username: &str) -> Vec<Collection> {
    let owner = CollectionOwner {
        username: "barforge".to_string(),
        display_name: Some("Barforge".to_string()),
        avatar_url: None,
    };
    vec![Collection {
        id: 1,
        user_id: 1,
        name: "Ops Essentials".to_string(),
        description: Some("Modules for steady-state ops dashboards.".to_string()),
        visibility: "public".to_string(),
        module_count: 4,
        owner,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-10T00:00:00Z".to_string(),
    }]
}

pub(crate) fn sample_collection(id: &str) -> Collection {
    let owner = CollectionOwner {
        username: "barforge".to_string(),
        display_name: Some("Barforge".to_string()),
        avatar_url: None,
    };
    let id_value = id.parse::<i64>().unwrap_or(0);
    if id == "ops-essentials" {
        Collection {
            id: id_value,
            user_id: 1,
            name: "Ops Essentials".to_string(),
            description: Some("Modules for steady-state ops dashboards.".to_string()),
            visibility: "public".to_string(),
            module_count: 4,
            owner,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-10T00:00:00Z".to_string(),
        }
    } else {
        Collection {
            id: id_value,
            user_id: 1,
            name: "Community Picks".to_string(),
            description: Some("Curated community favorites.".to_string()),
            visibility: "public".to_string(),
            module_count: 2,
            owner,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-10T00:00:00Z".to_string(),
        }
    }
}

pub(crate) fn sample_collection_modules() -> Vec<CollectionModule> {
    sample_modules()
        .into_iter()
        .enumerate()
        .map(|(index, module)| CollectionModule {
            uuid: module.uuid,
            name: module.name,
            author: module.author,
            category: module.category,
            note: None,
            position: (index + 1) as i64,
            added_at: "2024-01-02T00:00:00Z".to_string(),
        })
        .collect()
}

pub(crate) fn sample_modules() -> Vec<RegistryModule> {
    vec![
        RegistryModule {
            uuid: "weather-wttr@barforge".to_string(),
            name: "Weather".to_string(),
            description: "Weather forecast module".to_string(),
            author: "Barforge".to_string(),
            category: ModuleCategory::Weather,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/barforge/weather-wttr".to_string(),
            downloads: 3200,
            version: Some("1.3.2".to_string()),
            last_updated: Some("2025-11-05T12:00:00Z".to_string()),
            rating: Some(4.1),
            verified_author: true,
            tags: vec!["weather".to_string(), "status".to_string()],
            checksum: None,
            license: Some("MIT".to_string()),
        },
        RegistryModule {
            uuid: "clock-time@barforge".to_string(),
            name: "Clock".to_string(),
            description: "Minimal clock and timezone module".to_string(),
            author: "Barforge".to_string(),
            category: ModuleCategory::Time,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/barforge/clock-time".to_string(),
            downloads: 1800,
            version: Some("2.0.1".to_string()),
            last_updated: Some("2025-12-20T12:00:00Z".to_string()),
            rating: Some(4.8),
            verified_author: true,
            tags: vec!["clock".to_string(), "time".to_string()],
            checksum: None,
            license: Some("GPL-3.0".to_string()),
        },
        RegistryModule {
            uuid: "cpu-monitor@barforge".to_string(),
            name: "CPU Monitor".to_string(),
            description: "Real-time CPU usage module".to_string(),
            author: "OpsLab".to_string(),
            category: ModuleCategory::System,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/barforge/cpu-monitor".to_string(),
            downloads: 900,
            version: Some("0.9.4".to_string()),
            last_updated: Some("2025-10-12T08:30:00Z".to_string()),
            rating: Some(4.0),
            verified_author: false,
            tags: vec!["system".to_string(), "cpu".to_string()],
            checksum: None,
            license: Some("MIT".to_string()),
        },
    ]
}

pub(crate) fn sample_module_by_uuid(uuid: &str) -> RegistryModule {
    let mut modules = sample_modules();
    if let Some(found) = modules.iter().find(|module| module.uuid == uuid).cloned() {
        return found;
    }
    modules.pop().unwrap_or_else(sample_fallback_module)
}

pub(crate) fn sample_related_modules(exclude_uuid: &str) -> Vec<RegistryModule> {
    sample_modules()
        .into_iter()
        .filter(|module| module.uuid != exclude_uuid)
        .collect()
}

pub(crate) fn sample_versions() -> Vec<VersionHistoryEntry> {
    vec![
        VersionHistoryEntry {
            version: "1.3.2".to_string(),
            changelog: Some("Added Mocha theme support.".to_string()),
            downloads: 1200,
            published_at: "2025-12-01T12:00:00Z".to_string(),
        },
        VersionHistoryEntry {
            version: "1.3.1".to_string(),
            changelog: Some("Improved forecast refresh cadence.".to_string()),
            downloads: 980,
            published_at: "2025-10-20T12:00:00Z".to_string(),
        },
    ]
}

pub(crate) fn sample_screenshots() -> Vec<String> {
    vec![
        format!("{}", asset!("/assets/screenshots/module-screenshot-1.svg")),
        format!("{}", asset!("/assets/screenshots/module-screenshot-2.svg")),
    ]
}

pub(crate) fn sample_fallback_module() -> RegistryModule {
    RegistryModule {
        uuid: "module-fallback@barforge".to_string(),
        name: "Module".to_string(),
        description: "Fallback module".to_string(),
        author: "Barforge".to_string(),
        category: ModuleCategory::Custom,
        icon: None,
        screenshot: None,
        repo_url: "https://github.com/barforge".to_string(),
        downloads: 0,
        version: None,
        last_updated: None,
        rating: None,
        verified_author: false,
        tags: vec![],
        checksum: None,
        license: None,
    }
}
