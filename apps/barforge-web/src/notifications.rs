use barforge_types::NotificationType;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::api;
#[cfg(any(test, target_arch = "wasm32"))]
use barforge_types::Notification;
#[cfg(target_arch = "wasm32")]
use barforge_types::NotificationPreferences;
#[cfg(target_arch = "wasm32")]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{Event, EventSource, MessageEvent};

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "barforge:notification-preferences";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotificationIcon {
    Download,
    Comment,
    Star,
    Update,
    Announce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct NotificationPreference {
    pub(crate) kind: NotificationType,
    pub(crate) email: bool,
    pub(crate) in_app: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotificationStatus {
    Unread,
    Read,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NotificationItem {
    pub(crate) id: i64,
    pub(crate) kind: NotificationType,
    pub(crate) title: String,
    pub(crate) body: Option<String>,
    pub(crate) action_url: Option<String>,
    pub(crate) created_at: String,
    pub(crate) status: NotificationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveStatus {
    Saved,
    Error,
}

#[cfg(not(target_arch = "wasm32"))]
fn touch_save_status_variants() {
    let _ = SaveStatus::Saved;
    let _ = SaveStatus::Error;
}

#[derive(Clone)]
pub(crate) struct NotificationsStore {
    preferences: Signal<Vec<NotificationPreference>>,
    notifications: Signal<Vec<NotificationItem>>,
    #[cfg(target_arch = "wasm32")]
    syncing: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    stream: Signal<Option<EventSource>>,
    #[cfg(target_arch = "wasm32")]
    stream_connected: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    stream_error: Signal<Option<String>>,
    saving: Signal<bool>,
    status: Signal<Option<SaveStatus>>,
    authenticated: Signal<bool>,
    #[cfg(target_arch = "wasm32")]
    last_sync: Signal<Option<Instant>>,
}

#[cfg(target_arch = "wasm32")]
pub(crate) const NOTIFICATION_TYPES: [NotificationType; 5] = [
    NotificationType::Downloads,
    NotificationType::Comments,
    NotificationType::Stars,
    NotificationType::Updates,
    NotificationType::Announcements,
];

pub(crate) fn notification_label(kind: NotificationType) -> &'static str {
    match kind {
        NotificationType::Downloads => "Download milestones",
        NotificationType::Comments => "New comments",
        NotificationType::Stars => "New stars",
        NotificationType::Updates => "Module updates",
        NotificationType::Announcements => "Platform announcements",
    }
}

pub(crate) fn notification_key(kind: NotificationType) -> &'static str {
    match kind {
        NotificationType::Downloads => "downloads",
        NotificationType::Comments => "comments",
        NotificationType::Stars => "stars",
        NotificationType::Updates => "updates",
        NotificationType::Announcements => "announcements",
    }
}

pub(crate) fn notification_description(kind: NotificationType) -> &'static str {
    match kind {
        NotificationType::Downloads => {
            "Get notified when your modules reach download milestones (100, 1k, 10k, etc.)"
        }
        NotificationType::Comments => "When someone comments on your modules",
        NotificationType::Stars => "When someone stars your modules",
        NotificationType::Updates => "When modules you starred release new versions",
        NotificationType::Announcements => "Important updates about Barforge",
    }
}

pub(crate) fn notification_icon(kind: NotificationType) -> NotificationIcon {
    match kind {
        NotificationType::Downloads => NotificationIcon::Download,
        NotificationType::Comments => NotificationIcon::Comment,
        NotificationType::Stars => NotificationIcon::Star,
        NotificationType::Updates => NotificationIcon::Update,
        NotificationType::Announcements => NotificationIcon::Announce,
    }
}

pub(crate) fn use_notifications_provider() -> NotificationsStore {
    #[cfg(not(target_arch = "wasm32"))]
    touch_save_status_variants();

    let preferences = use_signal(load_preferences);
    let notifications = use_signal(Vec::new);
    #[cfg(target_arch = "wasm32")]
    let syncing = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let stream = use_signal(|| None::<EventSource>);
    #[cfg(target_arch = "wasm32")]
    let stream_connected = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let stream_error = use_signal(|| None::<String>);
    let saving = use_signal(|| false);
    let status = use_signal(|| None::<SaveStatus>);
    let authenticated = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let last_sync = use_signal(|| None::<Instant>);

    let store = NotificationsStore {
        preferences,
        notifications,
        #[cfg(target_arch = "wasm32")]
        syncing,
        #[cfg(target_arch = "wasm32")]
        stream,
        #[cfg(target_arch = "wasm32")]
        stream_connected,
        #[cfg(target_arch = "wasm32")]
        stream_error,
        saving,
        status,
        authenticated,
        #[cfg(target_arch = "wasm32")]
        last_sync,
    };

    use_effect({
        let preferences = store.preferences;
        move || {
            persist_preferences(&preferences.read());
        }
    });

    use_context_provider(|| store.clone());

    store
}

pub(crate) fn use_notifications() -> NotificationsStore {
    use_context::<NotificationsStore>()
}

impl NotificationsStore {
    pub(crate) fn preferences(&self) -> Vec<NotificationPreference> {
        self.preferences.read().clone()
    }

    pub(crate) fn notifications(&self) -> Vec<NotificationItem> {
        self.notifications.read().clone()
    }

    pub(crate) fn unread_count(&self) -> usize {
        unread_count(&self.notifications.read())
    }

    #[cfg(test)]
    pub(crate) fn set_notifications(&self, notifications: Vec<NotificationItem>) {
        set_signal(self.notifications, notifications);
    }

    pub(crate) fn saving(&self) -> bool {
        *self.saving.read()
    }

    pub(crate) fn status(&self) -> Option<SaveStatus> {
        *self.status.read()
    }

    pub(crate) fn set_authenticated(&self, value: bool) {
        let was_authenticated = *self.authenticated.read();
        if was_authenticated == value {
            return;
        }
        set_signal(self.authenticated, value);
        if value {
            self.sync_preferences();
        }
    }

    pub(crate) fn update_preference(
        &self,
        kind: NotificationType,
        field: PreferenceField,
        value: bool,
    ) {
        let mut next = self.preferences.read().clone();
        if let Some(pref) = next.iter_mut().find(|pref| pref.kind == kind) {
            match field {
                PreferenceField::Email => pref.email = value,
                PreferenceField::InApp => pref.in_app = value,
            }
        }
        set_signal(self.preferences, next);
        set_signal(self.status, None);
    }

    pub(crate) fn save_preferences(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if *self.saving.read() {
                return;
            }
            set_signal(self.saving, true);
            let store = self.clone();
            let payload = preferences_to_api(&store.preferences.read());
            spawn(async move {
                let result = api::update_notification_preferences(&payload).await;
                match result {
                    Ok(response) => {
                        let updated = preferences_from_api(&response.payload);
                        set_signal(store.preferences, updated);
                        set_signal(store.status, Some(SaveStatus::Saved));
                    }
                    Err(_) => {
                        set_signal(store.status, Some(SaveStatus::Error));
                    }
                }
                set_signal(store.saving, false);
            });
        }
    }

    pub(crate) fn sync_notifications(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if *self.syncing.read() || !api::LIVE_API_ENABLED {
                return;
            }
            set_signal(self.syncing, true);
            let store = self.clone();
            spawn(async move {
                if let Ok(payload) = api::fetch_notifications(20, 0).await {
                    let mapped = payload
                        .notifications
                        .iter()
                        .map(map_notification)
                        .collect::<Vec<_>>();
                    set_signal(store.notifications, mapped);
                }
                set_signal(store.syncing, false);
            });
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn connect_stream(&self) {
        let connected = *self.stream_connected.peek();
        let authenticated = *self.authenticated.peek();
        if !should_connect_stream(connected, authenticated) {
            return;
        }
        let source = match EventSource::new("/api/notifications/stream") {
            Ok(source) => source,
            Err(_) => {
                set_signal(self.stream_error, Some("Unable to connect".to_string()));
                return;
            }
        };

        let store = self.clone();
        let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().as_string() {
                match parse_notification_payload(&data) {
                    Ok(notification) => {
                        set_signal(store.stream_error, None);
                        store.ingest_notification(notification);
                    }
                    Err(message) => {
                        set_signal(store.stream_error, Some(message.to_string()));
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        let store = self.clone();
        let on_notification = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().as_string() {
                match parse_notification_payload(&data) {
                    Ok(notification) => {
                        set_signal(store.stream_error, None);
                        store.ingest_notification(notification);
                    }
                    Err(message) => {
                        set_signal(store.stream_error, Some(message.to_string()));
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        let _ = source.add_event_listener_with_callback(
            "notification",
            on_notification.as_ref().unchecked_ref(),
        );
        on_notification.forget();

        let store = self.clone();
        let on_open = Closure::wrap(Box::new(move |_event: Event| {
            let status = stream_status_on_open();
            set_signal(store.stream_connected, status.connected);
            set_signal(
                store.stream_error,
                status.error.map(|message| message.to_string()),
            );
        }) as Box<dyn FnMut(Event)>);
        source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let store = self.clone();
        let on_error = Closure::wrap(Box::new(move |_event: Event| {
            let status = stream_status_on_error();
            set_signal(store.stream_connected, status.connected);
            set_signal(
                store.stream_error,
                status.error.map(|message| message.to_string()),
            );
        }) as Box<dyn FnMut(Event)>);
        source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        set_signal(self.stream, Some(source));
        set_signal(self.stream_connected, true);
        set_signal(self.stream_error, None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn connect_stream(&self) {}

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn disconnect_stream(&self) {
        let stream = self.stream.peek().clone();
        let connected = *self.stream_connected.peek();
        let has_error = self.stream_error.peek().is_some();
        if !should_disconnect_stream(stream.is_some(), connected, has_error) {
            return;
        }
        if let Some(source) = stream {
            source.close();
        }
        set_signal(self.stream, None);
        set_signal(self.stream_connected, false);
        set_signal(self.stream_error, None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn disconnect_stream(&self) {}

    pub(crate) fn mark_all_read_with_sync(&self) {
        let previous = self.notifications.read().clone();
        let updated = mark_all_read(&previous);
        set_signal(self.notifications, updated);

        #[cfg(target_arch = "wasm32")]
        {
            let store = self.clone();
            spawn(async move {
                if api::mark_all_notifications_read().await.is_err() {
                    set_signal(store.notifications, previous);
                }
            });
        }
    }

    pub(crate) fn mark_read_with_sync(&self, id: i64) {
        let previous = self.notifications.read().clone();
        let updated = mark_read(&previous, id);
        set_signal(self.notifications, updated);

        #[cfg(target_arch = "wasm32")]
        {
            let store = self.clone();
            spawn(async move {
                if api::mark_notification_read(id).await.is_err() {
                    set_signal(store.notifications, previous);
                }
            });
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn ingest_notification(&self, notification: Notification) {
        let incoming = map_notification(&notification);
        let mut next = self.notifications.read().clone();
        if next.iter().any(|item| item.id == incoming.id) {
            return;
        }
        next.insert(0, incoming);
        set_signal(self.notifications, next);
    }

    fn sync_preferences(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let recent = self
                .last_sync
                .read()
                .as_ref()
                .map(|instant| instant.elapsed().as_secs() < 30)
                .unwrap_or(false);
            if recent {
                return;
            }
            let store = self.clone();
            spawn(async move {
                if let Ok(payload) = api::fetch_notification_preferences().await {
                    let prefs = preferences_from_api(&payload.payload);
                    set_signal(store.preferences, prefs);
                }
                set_signal(store.last_sync, Some(Instant::now()));
            });
        }
    }
}

fn unread_count(notifications: &[NotificationItem]) -> usize {
    notifications
        .iter()
        .filter(|notification| notification.status == NotificationStatus::Unread)
        .count()
}

fn mark_all_read(notifications: &[NotificationItem]) -> Vec<NotificationItem> {
    notifications
        .iter()
        .map(|notification| {
            let mut next = notification.clone();
            if next.status == NotificationStatus::Unread {
                next.status = NotificationStatus::Read;
            }
            next
        })
        .collect()
}

fn mark_read(notifications: &[NotificationItem], id: i64) -> Vec<NotificationItem> {
    notifications
        .iter()
        .map(|notification| {
            let mut next = notification.clone();
            if next.id == id && next.status == NotificationStatus::Unread {
                next.status = NotificationStatus::Read;
            }
            next
        })
        .collect()
}

#[cfg(any(test, target_arch = "wasm32"))]
fn map_notification(notification: &Notification) -> NotificationItem {
    let status = if notification.is_read {
        NotificationStatus::Read
    } else {
        NotificationStatus::Unread
    };

    NotificationItem {
        id: notification.id,
        kind: notification.notification_type,
        title: notification.title.clone(),
        body: notification.body.clone(),
        action_url: notification.action_url.clone(),
        created_at: notification.created_at.clone(),
        status,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreferenceField {
    Email,
    InApp,
}

#[cfg(target_arch = "wasm32")]
fn preferences_from_api(api: &NotificationPreferences) -> Vec<NotificationPreference> {
    NOTIFICATION_TYPES
        .iter()
        .map(|kind| {
            let (email, in_app) = match kind {
                NotificationType::Downloads => (api.email_downloads, api.downloads_enabled),
                NotificationType::Comments => (api.email_comments, api.comments_enabled),
                NotificationType::Stars => (api.email_stars, api.stars_enabled),
                NotificationType::Updates => (api.email_updates, api.updates_enabled),
                NotificationType::Announcements => {
                    (api.email_announcements, api.announcements_enabled)
                }
            };
            NotificationPreference {
                kind: *kind,
                email,
                in_app,
            }
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn preferences_to_api(preferences: &[NotificationPreference]) -> NotificationPreferences {
    let mut api = NotificationPreferences {
        downloads_enabled: false,
        comments_enabled: false,
        stars_enabled: false,
        updates_enabled: false,
        announcements_enabled: false,
        email_downloads: false,
        email_comments: false,
        email_stars: false,
        email_updates: false,
        email_announcements: false,
    };

    for pref in preferences {
        match pref.kind {
            NotificationType::Downloads => {
                api.downloads_enabled = pref.in_app;
                api.email_downloads = pref.email;
            }
            NotificationType::Comments => {
                api.comments_enabled = pref.in_app;
                api.email_comments = pref.email;
            }
            NotificationType::Stars => {
                api.stars_enabled = pref.in_app;
                api.email_stars = pref.email;
            }
            NotificationType::Updates => {
                api.updates_enabled = pref.in_app;
                api.email_updates = pref.email;
            }
            NotificationType::Announcements => {
                api.announcements_enabled = pref.in_app;
                api.email_announcements = pref.email;
            }
        }
    }

    api
}

#[cfg(target_arch = "wasm32")]
fn load_preferences() -> Vec<NotificationPreference> {
    local_storage_value(STORAGE_KEY)
        .and_then(|value| serde_json::from_str::<Vec<NotificationPreference>>(&value).ok())
        .unwrap_or_else(default_preferences)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_preferences() -> Vec<NotificationPreference> {
    default_preferences()
}

#[cfg(target_arch = "wasm32")]
fn persist_preferences(preferences: &[NotificationPreference]) {
    if let (Some(storage), Ok(payload)) = (local_storage(), serde_json::to_string(preferences)) {
        let _ = storage.set_item(STORAGE_KEY, &payload);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_preferences(_: &[NotificationPreference]) {}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn local_storage_value(key: &str) -> Option<String> {
    local_storage()?.get_item(key).ok().flatten()
}

fn default_preferences() -> Vec<NotificationPreference> {
    vec![
        NotificationPreference {
            kind: NotificationType::Downloads,
            email: false,
            in_app: true,
        },
        NotificationPreference {
            kind: NotificationType::Comments,
            email: false,
            in_app: true,
        },
        NotificationPreference {
            kind: NotificationType::Stars,
            email: false,
            in_app: true,
        },
        NotificationPreference {
            kind: NotificationType::Updates,
            email: false,
            in_app: true,
        },
        NotificationPreference {
            kind: NotificationType::Announcements,
            email: true,
            in_app: true,
        },
    ]
}

pub(crate) fn preferences_or_default(
    preferences: Vec<NotificationPreference>,
) -> Vec<NotificationPreference> {
    if preferences.is_empty() {
        default_preferences()
    } else {
        preferences
    }
}

fn set_signal<T: 'static>(signal: Signal<T>, value: T) {
    let mut signal = signal;
    signal.set(value);
}

#[cfg(any(test, target_arch = "wasm32"))]
const STREAM_INVALID_PAYLOAD: &str = "Invalid notification payload";

#[cfg(any(test, target_arch = "wasm32"))]
const STREAM_CONNECTION_LOST: &str = "Connection lost";

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StreamStatusUpdate {
    connected: bool,
    error: Option<&'static str>,
}

#[cfg(any(test, target_arch = "wasm32"))]
fn stream_status_on_open() -> StreamStatusUpdate {
    StreamStatusUpdate {
        connected: true,
        error: None,
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn stream_status_on_error() -> StreamStatusUpdate {
    StreamStatusUpdate {
        connected: false,
        error: Some(STREAM_CONNECTION_LOST),
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_notification_payload(data: &str) -> Result<Notification, &'static str> {
    serde_json::from_str::<Notification>(data).map_err(|_| STREAM_INVALID_PAYLOAD)
}

#[cfg(any(test, target_arch = "wasm32"))]
fn should_disconnect_stream(has_stream: bool, connected: bool, has_error: bool) -> bool {
    has_stream || connected || has_error
}

#[cfg(any(test, target_arch = "wasm32"))]
fn should_connect_stream(connected: bool, authenticated: bool) -> bool {
    !connected && authenticated
}

#[cfg(test)]
mod tests {
    use super::*;
    use barforge_types::Notification;

    fn sample_notification(id: i64, status: NotificationStatus) -> NotificationItem {
        NotificationItem {
            id,
            kind: NotificationType::Updates,
            title: "Update".to_string(),
            body: Some("New release".to_string()),
            action_url: Some("/modules/clock-time@barforge".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            status,
        }
    }

    #[test]
    fn unread_count_counts_only_unread() {
        let notifications = vec![
            sample_notification(1, NotificationStatus::Unread),
            sample_notification(2, NotificationStatus::Read),
            sample_notification(3, NotificationStatus::Done),
        ];

        assert_eq!(unread_count(&notifications), 1);
    }

    #[test]
    fn mark_all_read_preserves_done() {
        let notifications = vec![
            sample_notification(1, NotificationStatus::Unread),
            sample_notification(2, NotificationStatus::Done),
        ];
        let updated = mark_all_read(&notifications);

        assert_eq!(updated[0].status, NotificationStatus::Read);
        assert_eq!(updated[1].status, NotificationStatus::Done);
    }

    #[test]
    fn map_notification_sets_status_from_api() {
        let api = Notification {
            id: 99,
            notification_type: NotificationType::Stars,
            title: "New star".to_string(),
            body: None,
            action_url: Some("/modules/clock-time@barforge".to_string()),
            is_read: false,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let mapped = map_notification(&api);

        assert_eq!(mapped.id, 99);
        assert_eq!(mapped.status, NotificationStatus::Unread);
        assert_eq!(mapped.kind, NotificationType::Stars);
    }

    #[test]
    fn disconnect_stream_noops_when_already_cleared() {
        assert!(!should_disconnect_stream(false, false, false));
    }

    #[test]
    fn disconnect_stream_runs_when_any_state_is_set() {
        assert!(should_disconnect_stream(true, false, false));
        assert!(should_disconnect_stream(false, true, false));
        assert!(should_disconnect_stream(false, false, true));
    }

    #[test]
    fn connect_stream_requires_authentication() {
        assert!(!should_connect_stream(false, false));
    }

    #[test]
    fn connect_stream_skips_when_already_connected() {
        assert!(!should_connect_stream(true, true));
    }

    #[test]
    fn connect_stream_runs_when_authenticated_and_disconnected() {
        assert!(should_connect_stream(false, true));
    }

    #[test]
    fn preferences_or_default_returns_defaults_when_empty() {
        let prefs = preferences_or_default(Vec::new());

        assert_eq!(prefs.len(), default_preferences().len());
    }

    #[test]
    fn preferences_or_default_preserves_non_empty() {
        let prefs = vec![NotificationPreference {
            kind: NotificationType::Downloads,
            email: false,
            in_app: true,
        }];

        let result = preferences_or_default(prefs.clone());

        assert_eq!(result, prefs);
    }

    #[test]
    fn parse_notification_payload_returns_error_for_invalid_json() {
        let result = parse_notification_payload("{bad-json");

        assert_eq!(result, Err(STREAM_INVALID_PAYLOAD));
    }

    #[test]
    fn parse_notification_payload_parses_valid_json() {
        let notification = Notification {
            id: 7,
            notification_type: NotificationType::Stars,
            title: "New star".to_string(),
            body: Some("Nice work".to_string()),
            action_url: Some("/modules/test".to_string()),
            is_read: false,
            created_at: "2024-03-01T00:00:00Z".to_string(),
        };
        let payload = serde_json::to_string(&notification).expect("serialize notification");

        let parsed = parse_notification_payload(&payload).expect("parse notification payload");

        assert_eq!(parsed.id, 7);
        assert_eq!(parsed.notification_type, NotificationType::Stars);
    }

    #[test]
    fn stream_status_on_open_marks_connected_without_error() {
        let status = stream_status_on_open();

        assert!(status.connected);
        assert_eq!(status.error, None);
    }

    #[test]
    fn stream_status_on_error_marks_disconnected_with_message() {
        let status = stream_status_on_error();

        assert!(!status.connected);
        assert_eq!(status.error, Some(STREAM_CONNECTION_LOST));
    }
}
