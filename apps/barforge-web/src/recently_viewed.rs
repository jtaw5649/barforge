use barforge_types::{ModuleCategory, RegistryModule};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "recently_viewed_modules";
const MAX_RECENT: usize = 10;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecentModule {
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) author: String,
    pub(crate) description: String,
    pub(crate) category: ModuleCategory,
    pub(crate) downloads: i64,
    pub(crate) verified_author: bool,
    pub(crate) version: Option<String>,
    pub(crate) viewed_at: i64,
}

#[derive(Clone)]
pub(crate) struct RecentlyViewedStore {
    modules: Signal<Vec<RecentModule>>,
}

pub(crate) fn use_recently_viewed_provider() -> RecentlyViewedStore {
    let modules = use_signal(load_recently_viewed);
    let store = RecentlyViewedStore { modules };

    use_effect({
        let modules = store.modules;
        move || {
            persist_recently_viewed(&modules.read());
        }
    });

    use_context_provider(|| store.clone());

    store
}

pub(crate) fn use_recently_viewed() -> RecentlyViewedStore {
    use_context::<RecentlyViewedStore>()
}

impl RecentlyViewedStore {
    pub(crate) fn items(&self) -> Vec<RecentModule> {
        self.modules.read().clone()
    }

    pub(crate) fn add_from_module(&self, module: &RegistryModule) {
        self.add(RecentModule {
            uuid: module.uuid.clone(),
            name: module.name.clone(),
            author: module.author.clone(),
            description: module.description.clone(),
            category: module.category.clone(),
            downloads: module.downloads,
            verified_author: module.verified_author,
            version: module.version.clone(),
            viewed_at: now_millis(),
        });
    }

    pub(crate) fn add(&self, module: RecentModule) {
        if !is_valid_uuid(&module.uuid) {
            return;
        }
        let mut next = Vec::with_capacity(MAX_RECENT);
        next.push(module);
        for existing in self.modules.read().iter() {
            if existing.uuid == next[0].uuid {
                continue;
            }
            next.push(existing.clone());
            if next.len() == MAX_RECENT {
                break;
            }
        }
        let mut modules = self.modules;
        modules.set(next);
    }

    pub(crate) fn clear(&self) {
        let mut modules = self.modules;
        modules.set(Vec::new());
    }
}

impl RecentModule {
    pub(crate) fn to_registry_module(&self) -> RegistryModule {
        RegistryModule {
            uuid: self.uuid.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            category: self.category.clone(),
            icon: None,
            screenshot: None,
            repo_url: String::new(),
            downloads: self.downloads,
            version: self.version.clone(),
            last_updated: None,
            rating: None,
            verified_author: self.verified_author,
            tags: Vec::new(),
            checksum: None,
            license: None,
        }
    }
}

fn is_valid_uuid(value: &str) -> bool {
    is_uuid_like(value) || is_slug_like(value)
}

fn is_uuid_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, ch) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *ch != b'-' {
                    return false;
                }
            }
            _ => {
                if !ch.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

fn is_slug_like(value: &str) -> bool {
    let Some((name, owner)) = value.split_once('@') else {
        return false;
    };
    !name.is_empty() && !owner.is_empty()
}

#[cfg(target_arch = "wasm32")]
fn now_millis() -> i64 {
    js_sys::Date::now() as i64
}

#[cfg(not(target_arch = "wasm32"))]
fn now_millis() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp() * 1000
}

#[cfg(target_arch = "wasm32")]
fn load_recently_viewed() -> Vec<RecentModule> {
    let Some(payload) = local_storage_value(STORAGE_KEY) else {
        return Vec::new();
    };
    let parsed: Vec<RecentModule> = serde_json::from_str(&payload).unwrap_or_default();
    parsed
        .into_iter()
        .filter(|module| is_valid_uuid(&module.uuid))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_recently_viewed() -> Vec<RecentModule> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
fn persist_recently_viewed(modules: &[RecentModule]) {
    if let (Some(storage), Ok(payload)) = (local_storage(), serde_json::to_string(modules)) {
        let _ = storage.set_item(STORAGE_KEY, &payload);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_recently_viewed(_modules: &[RecentModule]) {}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn local_storage_value(key: &str) -> Option<String> {
    local_storage()?.get_item(key).ok().flatten()
}
