use barforge_types::{ModuleCategory, RegistryModule, ReviewsResponse};
use semver::Version;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct BrowseState {
    pub search_query: String,
    pub pending_search: Option<String>,
    pub search_debounce_start: Option<Instant>,
    pub selected_category: Option<ModuleCategory>,
    pub verified_only: bool,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub view_mode: ViewMode,
    pub persisted_view_mode: ViewMode,
    pub refreshing: bool,
    pub last_refreshed: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortField {
    #[default]
    Name,
    Downloads,
    RecentlyUpdated,
    Rating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    Ascending,
    #[default]
    Descending,
}

impl SortOrder {
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Cards,
    Table,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScreenshotState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(String),
    Failed,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ReviewsLoadingState {
    #[default]
    NotLoaded,
    Loading,
    Loaded(ReviewsResponse),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStage {
    RevocationCheck,
    FetchingSignature,
    DownloadingPackage,
    VerifyingSignature,
    VerifyingHash,
    ExtractingPackage,
    CheckingDependencies,
    RunningInstallScript,
    Complete,
}

impl InstallStage {
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::RevocationCheck => "Checking revocation status",
            Self::FetchingSignature => "Fetching signature",
            Self::DownloadingPackage => "Downloading package",
            Self::VerifyingSignature => "Verifying signature",
            Self::VerifyingHash => "Verifying hash",
            Self::ExtractingPackage => "Extracting package",
            Self::CheckingDependencies => "Checking dependencies",
            Self::RunningInstallScript => "Running install script",
            Self::Complete => "Installation complete",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModuleDetailState {
    pub screenshot: ScreenshotState,
    pub installing: bool,
    pub install_stage: Option<InstallStage>,
    pub reviews: ReviewsLoadingState,
}

impl ModuleDetailState {
    pub fn begin_screenshot_load(&mut self, has_screenshot: bool) {
        self.screenshot = if has_screenshot {
            ScreenshotState::Loading
        } else {
            ScreenshotState::NotLoaded
        };
    }

    pub fn handle_screenshot_loaded(&mut self, result: Result<String, ()>) {
        self.screenshot = match result {
            Ok(value) => ScreenshotState::Loaded(value),
            Err(()) => ScreenshotState::Failed,
        };
    }

    pub fn begin_reviews_load(&mut self) {
        self.reviews = ReviewsLoadingState::Loading;
    }

    pub fn handle_reviews_loaded(&mut self, result: Result<ReviewsResponse, String>) {
        self.reviews = match result {
            Ok(reviews) => ReviewsLoadingState::Loaded(reviews),
            Err(error) => ReviewsLoadingState::Failed(error),
        };
    }

    pub fn start_install(&mut self) {
        self.installing = true;
    }

    pub fn set_install_stage(&mut self, stage: InstallStage) {
        self.install_stage = Some(stage);
    }

    pub fn finish_install(&mut self, success: bool) {
        self.installing = false;
        if !success {
            self.install_stage = None;
        }
    }

    pub fn reset(&mut self) {
        self.screenshot = ScreenshotState::NotLoaded;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstalledModule {
    pub name: String,
    pub installed_version: Version,
    pub registry_version: Option<Version>,
}

impl InstalledModule {
    #[must_use]
    pub fn has_update(&self) -> bool {
        self.registry_version
            .as_ref()
            .is_some_and(|version| version > &self.installed_version)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstalledState {
    pub modules: Vec<InstalledModule>,
}

impl InstalledState {
    #[must_use]
    pub fn modules_with_updates(&self) -> Vec<&InstalledModule> {
        self.modules
            .iter()
            .filter(|module| module.has_update())
            .collect()
    }

    #[must_use]
    pub fn update_count(&self) -> usize {
        self.modules
            .iter()
            .filter(|module| module.has_update())
            .count()
    }
}

impl BrowseState {
    pub fn queue_search(&mut self, query: String, now: Instant) {
        self.pending_search = Some(query);
        self.search_debounce_start = Some(now);
    }

    pub fn apply_debounced_searches_at(&mut self, now: Instant) {
        const DEBOUNCE_MS: u64 = 150;

        let Some(start) = self.search_debounce_start else {
            return;
        };

        if now.saturating_duration_since(start) < Duration::from_millis(DEBOUNCE_MS) {
            return;
        }

        if let Some(query) = self.pending_search.take() {
            self.search_query = query;
        }

        self.search_debounce_start = None;
    }

    pub fn search_display(&self) -> &str {
        self.pending_search.as_deref().unwrap_or(&self.search_query)
    }

    pub fn start_refresh(&mut self) {
        self.refreshing = true;
    }

    pub fn finish_refresh(&mut self, success: bool, now: Instant) {
        self.refreshing = false;
        if success {
            self.last_refreshed = Some(now);
        }
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn mark_view_mode_persisted(&mut self) {
        self.persisted_view_mode = self.view_mode;
    }

    pub fn view_mode_dirty(&self) -> bool {
        self.view_mode != self.persisted_view_mode
    }

    pub fn filtered_modules<'a>(&self, modules: &'a [RegistryModule]) -> Vec<&'a RegistryModule> {
        let query = self.search_query.trim();
        let category = self.selected_category.as_ref();
        let verified_only = self.verified_only;
        let mut filtered: Vec<_> = modules
            .iter()
            .filter(|module| matches_search(module, query))
            .filter(|module| category.is_none_or(|selected| &module.category == selected))
            .filter(|module| !verified_only || module.verified_author)
            .collect();

        filtered.sort_by(|a, b| {
            let cmp = match self.sort_field {
                SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortField::Downloads => a.downloads.cmp(&b.downloads),
                SortField::RecentlyUpdated => a.last_updated.cmp(&b.last_updated),
                SortField::Rating => {
                    let a_rating = a.rating.unwrap_or(0.0);
                    let b_rating = b.rating.unwrap_or(0.0);
                    a_rating
                        .partial_cmp(&b_rating)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            };

            match self.sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });

        filtered
    }
}

fn matches_search(module: &RegistryModule, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_lowercase();
    module.name.to_lowercase().contains(&query_lower)
        || module.description.to_lowercase().contains(&query_lower)
        || module.author.to_lowercase().contains(&query_lower)
        || module
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query_lower))
}
