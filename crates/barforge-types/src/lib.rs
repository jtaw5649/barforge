use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// RFC 9457 Problem Details error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiVersion {
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Versioned<T> {
    pub version: i64,
    #[serde(flatten)]
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicStats {
    pub total_modules: i64,
    pub total_downloads: i64,
    pub total_authors: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandingInstallMethod {
    pub id: String,
    pub label: String,
    pub description: String,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandingData {
    pub stats: PublicStats,
    pub install_methods: Vec<LandingInstallMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewUser {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Moderator,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bluesky_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor_url: Option<String>,
    pub verified_author: bool,
    pub role: UserRole,
    pub module_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub id: i64,
    pub rating: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub helpful_count: i64,
    pub user: ReviewUser,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewsResponse {
    pub reviews: Vec<Review>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: i64,
    pub r2_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    pub position: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotsResponse {
    pub screenshots: Vec<Screenshot>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotUploadData {
    pub id: i64,
    pub r2_key: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotUploadResponse {
    pub version: i64,
    pub data: ScreenshotUploadData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotDeleteData {
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotDeleteResponse {
    pub version: i64,
    pub data: ScreenshotDeleteData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleCategory {
    System,
    Hardware,
    Network,
    Audio,
    Power,
    Time,
    Workspace,
    Window,
    Tray,
    Weather,
    Productivity,
    Media,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryModule {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub category: ModuleCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    pub repo_url: String,
    pub downloads: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    pub verified_author: bool,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModulesResponse {
    pub modules: Vec<RegistryModule>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionOwner {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub visibility: String,
    pub module_count: i64,
    pub owner: CollectionOwner,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionsResponse {
    pub collections: Vec<Collection>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionModule {
    pub uuid: String,
    pub name: String,
    pub author: String,
    pub category: ModuleCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub position: i64,
    pub added_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionDetailResponse {
    pub collection: Collection,
    pub modules: Vec<CollectionModule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarredModule {
    #[serde(flatten)]
    pub module: RegistryModule,
    pub starred_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarStatusResponse {
    pub starred: bool,
    pub star_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarsResponse {
    pub modules: Vec<StarredModule>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncStarsRequest {
    pub uuids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncStarsResponse {
    pub synced: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Downloads,
    Comments,
    Stars,
    Updates,
    Announcements,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub notification_type: NotificationType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_url: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub downloads_enabled: bool,
    pub comments_enabled: bool,
    pub stars_enabled: bool,
    pub updates_enabled: bool,
    pub announcements_enabled: bool,
    pub email_downloads: bool,
    pub email_comments: bool,
    pub email_stars: bool,
    pub email_updates: bool,
    pub email_announcements: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<Notification>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnreadCountResponse {
    pub unread_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkReadResponse {
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkAllReadResponse {
    pub marked_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_modules: i64,
    pub total_users: i64,
    pub total_downloads: i64,
    pub pending_submissions: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectRequest {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub user_id: i64,
    pub verified_author: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Submission {
    pub id: i64,
    pub submitter_id: i64,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub repo_url: String,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub submitted_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<i64>,
    pub submitter_username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionsResponse {
    pub submissions: Vec<Submission>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturedModulesResponse {
    pub version: i64,
    pub featured: Vec<RegistryModule>,
    pub popular: Vec<RegistryModule>,
    pub recent: Vec<RegistryModule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistoryEntry {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    pub downloads: i64,
    pub published_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionsResponse {
    pub versions: Vec<VersionHistoryEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub version: i64,
    pub modules: Vec<RegistryModule>,
    pub categories: BTreeMap<String, CategoryInfo>,
}

#[cfg(test)]
mod tests {
    use super::{LandingData, LandingInstallMethod, PublicStats, Versioned};

    #[test]
    fn landing_response_roundtrip() {
        let json = r#"{"version":1,"stats":{"total_modules":12,"total_downloads":3400,"total_authors":4},"install_methods":[{"id":"aur","label":"AUR","description":"Arch User Repository","commands":["yay -S barforge"]}]}"#;

        let parsed: Versioned<LandingData> = serde_json::from_str(json).expect("landing response");

        assert_eq!(
            parsed.payload.stats,
            PublicStats {
                total_modules: 12,
                total_downloads: 3400,
                total_authors: 4,
            }
        );
        assert_eq!(parsed.payload.install_methods.len(), 1);
        assert_eq!(
            parsed.payload.install_methods[0],
            LandingInstallMethod {
                id: "aur".to_string(),
                label: "AUR".to_string(),
                description: "Arch User Repository".to_string(),
                commands: vec!["yay -S barforge".to_string()],
            }
        );
    }
}
