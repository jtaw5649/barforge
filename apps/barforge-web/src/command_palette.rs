use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteItemKind {
    Page,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    CopyUrl,
    RefreshPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteIcon {
    Home,
    Download,
    External,
    Dashboard,
    Settings,
    Login,
    Copy,
    Refresh,
    Grid,
}

impl PaletteIcon {
    #[must_use]
    pub fn path(self) -> &'static str {
        match self {
            PaletteIcon::Home => {
                "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
            }
            PaletteIcon::Grid => {
                "M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
            }
            PaletteIcon::Download => {
                "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
            }
            PaletteIcon::Dashboard => {
                "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            }
            PaletteIcon::Settings => {
                "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            }
            PaletteIcon::Login => {
                "M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"
            }
            PaletteIcon::Copy => {
                "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
            }
            PaletteIcon::Refresh => {
                "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            }
            PaletteIcon::External => "M14 3h7m0 0v7m0-7L10 14m-1 7H5a2 2 0 01-2-2V5a2 2 0 012-2h4",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub kind: PaletteItemKind,
    pub icon: PaletteIcon,
    pub path: Option<&'static str>,
    pub action: Option<PaletteAction>,
    pub requires_auth: bool,
    pub requires_admin: bool,
    pub hide_when_auth: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    All,
    Pages,
    Commands,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteResult {
    pub item: PaletteItem,
    pub score: i32,
}

pub fn palette_items(is_authenticated: bool, is_admin: bool) -> Vec<PaletteItem> {
    let items = all_items();
    items
        .into_iter()
        .filter(|item| {
            if item.hide_when_auth && is_authenticated {
                return false;
            }
            if item.requires_auth && !is_authenticated {
                return false;
            }
            if item.requires_admin && !is_admin {
                return false;
            }
            true
        })
        .collect()
}

pub fn filter_palette(items: &[PaletteItem], query: &str, mode: PaletteMode) -> Vec<PaletteResult> {
    let filtered = items
        .iter()
        .filter(|item| match mode {
            PaletteMode::All => true,
            PaletteMode::Pages => item.kind == PaletteItemKind::Page,
            PaletteMode::Commands => item.kind == PaletteItemKind::Command,
        })
        .filter_map(|item| {
            let score = match_score(item.name, query)
                .or_else(|| match_score(item.description, query))
                .unwrap_or_else(|| if query.trim().is_empty() { 0 } else { -1 });
            if score < 0 {
                return None;
            }
            Some(PaletteResult {
                item: item.clone(),
                score,
            })
        })
        .collect::<Vec<_>>();

    let mut results = filtered;
    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[derive(Clone)]
pub struct PaletteState {
    open: Signal<bool>,
    mode: Signal<PaletteMode>,
    query: Signal<String>,
}

impl PaletteState {
    #[must_use]
    pub fn from_signals(
        open: Signal<bool>,
        mode: Signal<PaletteMode>,
        query: Signal<String>,
    ) -> Self {
        Self { open, mode, query }
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        *self.open.read()
    }

    pub fn open(&self) {
        let mut open = self.open;
        open.set(true);
    }

    pub fn close(&self) {
        let mut open = self.open;
        open.set(false);
    }

    pub fn toggle(&self) {
        let next = !self.is_open();
        let mut open = self.open;
        open.set(next);
    }

    pub fn set_mode(&self, mode: PaletteMode) {
        let mut stored = self.mode;
        stored.set(mode);
    }

    #[must_use]
    pub fn mode(&self) -> PaletteMode {
        *self.mode.read()
    }

    pub fn set_query(&self, value: String) {
        let mut query = self.query;
        query.set(value);
    }

    #[must_use]
    pub fn query(&self) -> String {
        self.query.read().clone()
    }
}

fn match_score(text: &str, query: &str) -> Option<i32> {
    let trimmed = query.trim().to_lowercase();
    if trimmed.is_empty() {
        return Some(0);
    }
    let haystack = text.to_lowercase();
    if let Some(pos) = haystack.find(&trimmed) {
        return Some(1000 - pos as i32);
    }
    let mut last_index = 0usize;
    let mut gaps = 0i32;
    for ch in trimmed.chars() {
        if let Some(idx) = haystack[last_index..].find(ch) {
            gaps += idx as i32;
            last_index += idx + 1;
        } else {
            return None;
        }
    }
    Some(300 - gaps)
}

fn all_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            id: "home",
            name: "Home",
            description: "Go to homepage",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::Home,
            path: Some("/"),
            action: None,
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "get-started",
            name: "Get Started",
            description: "Install Barforge App",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::Download,
            path: Some("https://github.com/jtaw5649/barforge-app"),
            action: None,
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "docs",
            name: "Waybar Docs",
            description: "Read the official Waybar docs",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::External,
            path: Some("https://github.com/Alexays/Waybar"),
            action: None,
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "dashboard",
            name: "Dashboard",
            description: "Your personal dashboard",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::Dashboard,
            path: Some("/dashboard"),
            action: None,
            requires_auth: true,
            requires_admin: false,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "admin",
            name: "Admin Panel",
            description: "Administration settings",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::Settings,
            path: Some("/admin"),
            action: None,
            requires_auth: true,
            requires_admin: true,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "login",
            name: "Login",
            description: "Log in to your account",
            kind: PaletteItemKind::Page,
            icon: PaletteIcon::Login,
            path: Some("/login"),
            action: None,
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: true,
        },
        PaletteItem {
            id: "copy-url",
            name: "Copy Current URL",
            description: "Copy page URL to clipboard",
            kind: PaletteItemKind::Command,
            icon: PaletteIcon::Copy,
            path: None,
            action: Some(PaletteAction::CopyUrl),
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: false,
        },
        PaletteItem {
            id: "refresh",
            name: "Refresh Page",
            description: "Reload the current page",
            kind: PaletteItemKind::Command,
            icon: PaletteIcon::Refresh,
            path: None,
            action: Some(PaletteAction::RefreshPage),
            requires_auth: false,
            requires_admin: false,
            hide_when_auth: false,
        },
    ]
}
