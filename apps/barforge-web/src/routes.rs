use dioxus::prelude::*;
use dioxus_router::Routable;

#[allow(unused_imports)]
use crate::pages::{
    AdminRoute, BarforgeRoute, CollectionDetailRoute, DashboardRoute, Home, LoginRoute,
    ModuleDetailRoute, ModulesIndex, ModulesLayout, ModulesSearch, NotFoundRoute, PrivacyRoute,
    SettingsIndex, SettingsLayout, SettingsNotifications, SettingsProfile, SettingsSecurity,
    StarsRoute, TermsRoute, UploadRoute, UserProfileRoute,
};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[nest("/modules")]
    #[layout(ModulesLayout)]
    #[route("/")]
    ModulesIndex {},
    #[route("/search?:q&:category&:sort&:page&:view")]
    ModulesSearch {
        q: Option<String>,
        category: Option<String>,
        sort: Option<String>,
        page: Option<usize>,
        view: Option<String>,
    },
    #[route("/:uuid")]
    ModuleDetailRoute { uuid: ModuleSlug },
    #[end_layout]
    #[end_nest]
    #[nest("/settings")]
    #[layout(SettingsLayout)]
    #[route("/")]
    SettingsIndex {},
    #[route("/profile")]
    SettingsProfile {},
    #[route("/notifications")]
    SettingsNotifications {},
    #[route("/security")]
    SettingsSecurity {},
    #[end_layout]
    #[end_nest]
    #[route("/users/:username")]
    UserProfileRoute { username: GithubUsername },
    #[route("/collections/:id")]
    CollectionDetailRoute { id: CollectionId },
    #[route("/stars")]
    StarsRoute {},
    #[route("/upload")]
    UploadRoute {},
    #[route("/dashboard")]
    DashboardRoute {},
    #[route("/admin")]
    AdminRoute {},
    #[route("/barforge")]
    BarforgeRoute {},
    #[route("/terms")]
    TermsRoute {},
    #[route("/privacy")]
    PrivacyRoute {},
    #[route("/login?:redirect_to")]
    LoginRoute { redirect_to: Option<String> },
    #[route("/")]
    Home {},
    #[route("/:..segments")]
    NotFoundRoute { segments: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSlug(String);

impl ModuleSlug {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ModuleSlug {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for ModuleSlug {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (name, owner) = value.split_once('@').ok_or("missing owner")?;
        if !is_module_segment(name) {
            return Err("invalid module name");
        }
        if !is_github_username(owner) {
            return Err("invalid module owner");
        }
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GithubUsername(String);

impl GithubUsername {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GithubUsername {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for GithubUsername {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if is_github_username(value) {
            Ok(Self(value.to_string()))
        } else {
            Err("invalid username")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionId(String);

impl CollectionId {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CollectionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for CollectionId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if is_module_segment(value) {
            Ok(Self(value.to_string()))
        } else {
            Err("invalid collection id")
        }
    }
}

fn is_module_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    let Some(last) = value.chars().last() else {
        return false;
    };
    if !last.is_ascii_alphanumeric() {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
}

fn is_github_username(value: &str) -> bool {
    if value.is_empty() || value.len() > 39 {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    let Some(last) = value.chars().last() else {
        return false;
    };
    if !last.is_ascii_alphanumeric() {
        return false;
    }
    let mut prev_dash = false;
    for ch in value.chars() {
        if ch == '-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
            continue;
        }
        if !ch.is_ascii_alphanumeric() {
            return false;
        }
        prev_dash = false;
    }
    true
}
