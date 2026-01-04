use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    #[must_use]
    pub fn cycle(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }

    #[must_use]
    pub fn effective(self, system: ThemeMode) -> ThemeMode {
        match self {
            Self::System => system,
            Self::Light => ThemeMode::Light,
            Self::Dark => ThemeMode::Dark,
        }
    }

    #[must_use]
    pub fn from_storage(value: Option<&str>) -> Self {
        match value {
            Some("light") => Self::Light,
            Some("dark") => Self::Dark,
            Some("system") => Self::System,
            _ => Self::System,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Clone)]
pub struct ThemeController {
    current: Signal<ThemePreference>,
}

impl ThemeController {
    #[must_use]
    pub fn current(&self) -> ThemePreference {
        *self.current.read()
    }

    pub fn cycle(&self) {
        let next = self.current();
        let next = next.cycle();
        self.set(next);
    }

    pub fn set(&self, preference: ThemePreference) {
        let mut current = self.current;
        current.set(preference);
        apply_theme(preference);
    }
}

pub fn use_theme() -> ThemeController {
    let initial = load_theme();
    let current = use_signal(|| initial);
    let controller = ThemeController { current };

    use_effect({
        let controller = controller.clone();
        move || {
            let preference = controller.current();
            apply_theme(preference);
        }
    });

    controller
}

#[cfg(target_arch = "wasm32")]
fn load_theme() -> ThemePreference {
    let window = web_sys::window();
    let stored = window
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("theme").ok().flatten());
    ThemePreference::from_storage(stored.as_deref())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_theme() -> ThemePreference {
    ThemePreference::System
}

#[cfg(target_arch = "wasm32")]
fn apply_theme(preference: ThemePreference) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("theme", preference.as_str());
        }
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                let effective = preference.effective(system_theme());
                let _ = root.set_attribute(
                    "data-theme",
                    match effective {
                        ThemeMode::Light => "light",
                        ThemeMode::Dark => "dark",
                    },
                );
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_theme(_preference: ThemePreference) {}

#[cfg(target_arch = "wasm32")]
fn system_theme() -> ThemeMode {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
        })
        .map(|media| {
            if media.matches() {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            }
        })
        .unwrap_or(ThemeMode::Dark)
}
