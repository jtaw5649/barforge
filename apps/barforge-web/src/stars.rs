use std::collections::HashSet;
#[cfg(target_arch = "wasm32")]
use std::time::{Duration, Instant};

use barforge_types::RegistryModule;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::api;

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "starred_modules";
#[cfg(target_arch = "wasm32")]
const SYNC_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub(crate) struct StarsStore {
    starred: Signal<HashSet<String>>,
    cached_modules: Signal<Vec<RegistryModule>>,
    syncing: Signal<bool>,
    authenticated: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    last_sync: Signal<Option<Instant>>,
}

pub(crate) fn use_stars_provider() -> StarsStore {
    let starred = use_signal(load_stars);
    let cached_modules = use_signal(Vec::new);
    let syncing = use_signal(|| false);
    let authenticated = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let last_sync = use_signal(|| None::<Instant>);

    let store = StarsStore {
        starred,
        cached_modules,
        syncing,
        authenticated,
        #[cfg(target_arch = "wasm32")]
        last_sync,
    };

    use_effect({
        let starred = store.starred;
        move || {
            persist_stars(&starred.read());
        }
    });

    use_effect({
        let store = store.clone();
        move || {
            let authenticated = *store.authenticated.read();
            if !authenticated {
                let _ = store.starred.read().len();
                store.refresh_local_modules();
            }
        }
    });

    use_context_provider(|| store.clone());

    store
}

pub(crate) fn use_stars() -> StarsStore {
    use_context::<StarsStore>()
}

impl StarsStore {
    pub(crate) fn starred_set(&self) -> HashSet<String> {
        self.starred.read().clone()
    }

    pub(crate) fn cached_modules(&self) -> Vec<RegistryModule> {
        self.cached_modules.read().clone()
    }

    pub(crate) fn syncing(&self) -> bool {
        *self.syncing.read()
    }

    pub(crate) fn set_authenticated(&self, value: bool) {
        let was_authenticated = *self.authenticated.read();
        if was_authenticated == value {
            return;
        }
        set_signal(self.authenticated, value);
        if value {
            self.sync_with_server();
        } else {
            self.refresh_local_modules();
        }
    }

    pub(crate) fn toggle(&self, uuid: &str) {
        let was_starred = self.starred.read().contains(uuid);
        self.update_starred(uuid, !was_starred);

        if !*self.authenticated.read() {
            self.refresh_local_modules();
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                let uuid = uuid.to_string();
                let store = self.clone();
                spawn(async move {
                    let result = if was_starred {
                        api::unstar_module(&uuid).await
                    } else {
                        api::star_module(&uuid).await
                    };

                    let ok = result
                        .as_ref()
                        .map(|response| response.success)
                        .unwrap_or(false);
                    if !ok {
                        store.restore_star_state(&uuid, was_starred);
                    }
                });
            }
        }
    }

    fn sync_with_server(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if *self.syncing.read() {
                return;
            }
            let recent = self
                .last_sync
                .read()
                .as_ref()
                .map(|instant| instant.elapsed() < SYNC_INTERVAL)
                .unwrap_or(false);
            if recent {
                return;
            }

            set_signal(self.syncing, true);
            let store = self.clone();
            spawn(async move {
                if let Ok(payload) = api::fetch_stars().await {
                    let server_modules = payload.payload.modules;
                    let server_ids: HashSet<String> = server_modules
                        .iter()
                        .map(|module| module.module.uuid.clone())
                        .collect();
                    let mut merged = store.starred.read().clone();
                    for id in &server_ids {
                        merged.insert(id.clone());
                    }
                    set_signal(store.starred, merged.clone());
                    let modules = server_modules
                        .into_iter()
                        .map(|module| module.module)
                        .collect::<Vec<_>>();
                    set_signal(store.cached_modules, modules);

                    let local_only = merged
                        .into_iter()
                        .filter(|uuid| !server_ids.contains(uuid))
                        .collect::<Vec<_>>();
                    for uuid in local_only {
                        if let Ok(response) = api::star_module(&uuid).await {
                            if !response.success {
                                store.restore_star_state(&uuid, false);
                            }
                        }
                    }
                }
                set_signal(store.last_sync, Some(Instant::now()));
                set_signal(store.syncing, false);
            });
        }
    }

    fn refresh_local_modules(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if *self.syncing.read() {
                return;
            }
            let uuids = self.starred.read().iter().cloned().collect::<Vec<_>>();
            if uuids.is_empty() {
                let cached_empty = self.cached_modules.read().is_empty();
                if !should_clear_cached_modules(true, cached_empty) {
                    return;
                }
                set_signal(self.cached_modules, Vec::new());
                return;
            }
            set_signal(self.syncing, true);
            let store = self.clone();
            spawn(async move {
                let mut modules = Vec::new();
                for uuid in uuids {
                    if let Ok(module) = api::fetch_module_detail(&uuid).await {
                        modules.push(module);
                    }
                }
                set_signal(store.cached_modules, modules);
                set_signal(store.syncing, false);
            });
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn restore_star_state(&self, uuid: &str, was_starred: bool) {
        self.update_starred(uuid, was_starred);
    }

    fn update_starred(&self, uuid: &str, should_star: bool) {
        let mut next = self.starred.read().clone();
        if should_star {
            next.insert(uuid.to_string());
        } else {
            next.remove(uuid);
        }
        set_signal(self.starred, next);
    }
}

fn set_signal<T: 'static>(signal: Signal<T>, value: T) {
    let mut signal = signal;
    signal.set(value);
}

#[cfg(any(test, target_arch = "wasm32"))]
fn should_clear_cached_modules(starred_empty: bool, cached_empty: bool) -> bool {
    starred_empty && !cached_empty
}

#[cfg(target_arch = "wasm32")]
fn load_stars() -> HashSet<String> {
    local_storage_value(STORAGE_KEY)
        .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
        .unwrap_or_default()
        .into_iter()
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_stars() -> HashSet<String> {
    HashSet::new()
}

#[cfg(target_arch = "wasm32")]
fn persist_stars(stars: &HashSet<String>) {
    let values = stars.iter().cloned().collect::<Vec<_>>();
    if let (Some(storage), Ok(payload)) = (local_storage(), serde_json::to_string(&values)) {
        let _ = storage.set_item(STORAGE_KEY, &payload);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_stars(_: &HashSet<String>) {}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn local_storage_value(key: &str) -> Option<String> {
    local_storage()?.get_item(key).ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::should_clear_cached_modules;

    #[test]
    fn skips_clearing_cache_when_starred_and_cache_are_empty() {
        assert!(!should_clear_cached_modules(true, true));
    }

    #[test]
    fn clears_cache_when_starred_is_empty_but_cache_has_entries() {
        assert!(should_clear_cached_modules(true, false));
    }

    #[test]
    fn skips_clearing_cache_when_starred_has_entries() {
        assert!(!should_clear_cached_modules(false, false));
    }
}
