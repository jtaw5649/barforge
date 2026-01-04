#[cfg(feature = "server")]
mod auth_tests {
    use axum::Router;
    use axum::body::{Body, Bytes};
    use axum::extract::{Path, State};
    use axum::http::{Method, Request, StatusCode, header};
    use axum::response::{Json, Response};
    use axum::routing::{delete, get, post};
    use barforge_types::{
        AdminStats, Collection, CollectionDetailResponse, CollectionModule, CollectionOwner,
        CollectionsResponse, ModuleCategory, ModulesResponse, NotificationPreferences,
        RegistryModule, RejectRequest, ScreenshotDeleteData, ScreenshotDeleteResponse,
        ScreenshotUploadData, ScreenshotUploadResponse, StarStatusResponse, StarredModule,
        StarsResponse, Submission, SubmissionsResponse, SyncStarsRequest, SyncStarsResponse,
        UserProfile, UserRole, VerifyResponse, Versioned,
    };
    use barforge_web::server::auth;
    use barforge_web::server::{self, AuthState, ResendConfig};
    use http_body_util::BodyExt;
    use oauth2::url::Url;
    use reqwest::redirect::Policy;
    use serde::Deserialize;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    const TEST_APP_BASE_URL: &str = "http://localhost:8080";
    const TEST_PUBLIC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/public");
    const TEST_TOKEN_SECRET: &str = "test-token-secret";
    const TEST_DATE: &str = "2024-01-01T00:00:00Z";
    const TEST_CLIENT_ID: &str = "client-id";
    const TEST_CLIENT_SECRET: &str = "client-secret";

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        vars: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn new(set_vars: Vec<(String, String)>, unset_vars: Vec<String>) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
            let mut originals: HashMap<String, Option<String>> = HashMap::new();
            for key in set_vars
                .iter()
                .map(|(key, _)| key.clone())
                .chain(unset_vars.iter().cloned())
            {
                originals
                    .entry(key.clone())
                    .or_insert_with(|| std::env::var(&key).ok());
            }
            for (key, value) in set_vars {
                unsafe { std::env::set_var(key, value) };
            }
            for key in unset_vars {
                unsafe { std::env::remove_var(key) };
            }
            let vars = originals.into_iter().collect();
            Self { _lock: lock, vars }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.vars.drain(..) {
                match value {
                    Some(value) => unsafe { std::env::set_var(key, value) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }

    #[derive(Default)]
    struct CookieJar {
        cookies: HashMap<String, String>,
    }

    impl CookieJar {
        fn update(&mut self, response: &Response) {
            for value in response.headers().get_all(header::SET_COOKIE) {
                let Ok(value) = value.to_str() else {
                    continue;
                };
                let Some((name, value)) = value
                    .split(';')
                    .next()
                    .and_then(|pair| pair.split_once('='))
                else {
                    continue;
                };
                self.cookies.insert(name.to_string(), value.to_string());
            }
        }

        fn header_value(&self) -> Option<String> {
            if self.cookies.is_empty() {
                return None;
            }
            let mut parts = Vec::new();
            for (name, value) in &self.cookies {
                parts.push(format!("{name}={value}"));
            }
            Some(parts.join("; "))
        }
    }

    #[derive(Clone, Deserialize)]
    struct ResendEmailRequest {
        from: String,
        to: Vec<String>,
        subject: String,
        text: String,
    }

    #[derive(Clone)]
    struct MockState {
        github_login: String,
        github_email: String,
        export_emails: Arc<Mutex<Vec<ResendEmailRequest>>>,
        profile_role: UserRole,
        sync_calls: Arc<AtomicUsize>,
    }

    impl MockState {
        fn new(login: &str, role: UserRole) -> Self {
            Self {
                github_login: login.to_string(),
                github_email: format!("{login}@example.com"),
                export_emails: Arc::new(Mutex::new(Vec::new())),
                profile_role: role,
                sync_calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn record_export_email(&self, payload: ResendEmailRequest) {
            let mut emails = self
                .export_emails
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            emails.push(payload);
        }

        fn export_emails(&self) -> Vec<ResendEmailRequest> {
            self.export_emails
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .clone()
        }

        fn record_sync_call(&self) {
            self.sync_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn sync_calls(&self) -> usize {
            self.sync_calls.load(Ordering::SeqCst)
        }
    }

    struct MockServer {
        base_url: String,
        state: MockState,
    }

    impl MockServer {
        async fn start(login: &str, role: UserRole) -> Self {
            let state = MockState::new(login, role);
            let app = mock_router(state.clone());
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind mock server");
            let addr = listener.local_addr().expect("mock server addr");
            tokio::spawn(async move {
                axum::serve(listener, app).await.expect("serve mock server");
            });
            Self {
                base_url: format!("http://{addr}"),
                state,
            }
        }
    }

    struct TestHarness {
        app: Router,
        mock: MockServer,
        _env: EnvGuard,
    }

    impl TestHarness {
        async fn new() -> Self {
            Self::build("octo", UserRole::User, false).await
        }

        async fn new_with_user(login: &str) -> Self {
            Self::build(login, UserRole::User, false).await
        }

        async fn new_with_role(login: &str, role: UserRole) -> Self {
            Self::build(login, role, false).await
        }

        async fn new_with_turnstile(login: &str) -> Self {
            Self::build(login, UserRole::User, true).await
        }

        async fn build(login: &str, role: UserRole, turnstile: bool) -> Self {
            let mock = MockServer::start(login, role).await;
            let mut set_vars = vec![
                (
                    "DIOXUS_PUBLIC_PATH".to_string(),
                    TEST_PUBLIC_PATH.to_string(),
                ),
                ("BARFORGE_API_BASE_URL".to_string(), mock.base_url.clone()),
                ("PUBLIC_API_BASE_URL".to_string(), mock.base_url.clone()),
                (
                    "BARFORGE_RESEND_API_KEY".to_string(),
                    "re_test_key".to_string(),
                ),
                (
                    "BARFORGE_RESEND_FROM".to_string(),
                    "Barforge <noreply@barforge.dev>".to_string(),
                ),
                (
                    "BARFORGE_RESEND_BASE_URL".to_string(),
                    mock.base_url.clone(),
                ),
            ];
            let mut unset_vars = vec![
                "TURNSTILE_SECRET".to_string(),
                "TURNSTILE_VERIFY_URL".to_string(),
            ];
            if turnstile {
                set_vars.push(("TURNSTILE_SECRET".to_string(), "test-secret".to_string()));
                set_vars.push(("TURNSTILE_VERIFY_URL".to_string(), mock.base_url.clone()));
                unset_vars.clear();
            }
            let env = EnvGuard::new(set_vars, unset_vars);

            let auth_state = build_auth_state(&mock.base_url, vec!["admin".to_string()]);
            let env_fn = |key: &str| match key {
                "BARFORGE_SESSION_DB_URL" => Some("sqlite::memory:".to_string()),
                "BARFORGE_SESSION_SECURE" => Some("false".to_string()),
                _ => None,
            };
            let session_layer = server::session_layer_from_env(&env_fn)
                .await
                .expect("session layer");
            let app = server::app_router(auth_state, session_layer);

            Self {
                app,
                mock,
                _env: env,
            }
        }

        async fn request(&self, request: Request<Body>) -> Response {
            self.app
                .clone()
                .oneshot(request)
                .await
                .expect("app response")
        }

        fn export_emails(&self) -> Vec<ResendEmailRequest> {
            self.mock.state.export_emails()
        }

        fn sync_calls(&self) -> usize {
            self.mock.state.sync_calls()
        }
    }

    fn build_auth_state(base_url: &str, admin_logins: Vec<String>) -> AuthState {
        let redirect_url = format!("{TEST_APP_BASE_URL}/auth/github/callback");
        let github_client = auth::github_client_with_urls(
            TEST_CLIENT_ID,
            TEST_CLIENT_SECRET,
            &redirect_url,
            &format!("{base_url}/login/oauth/authorize"),
            &format!("{base_url}/login/oauth/access_token"),
        )
        .expect("github client");
        let github_api_base = Url::parse(&format!("{base_url}/")).expect("github api base");
        let http_client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("http client");
        let token_key = auth::token_key_from_secret(TEST_TOKEN_SECRET);
        let resend = ResendConfig {
            api_key: Some("re_test_key".to_string()),
            from: Some("Barforge <noreply@barforge.dev>".to_string()),
            base_url: Url::parse(base_url).expect("resend base"),
        };
        AuthState::new_with_admins(
            github_client,
            github_api_base,
            http_client,
            admin_logins,
            token_key,
            resend,
        )
    }

    #[derive(Deserialize)]
    struct CsrfTokenResponse {
        token: String,
    }

    #[derive(Deserialize)]
    struct SessionUser {
        login: String,
        email: Option<String>,
    }

    #[derive(Deserialize)]
    struct SessionResponse {
        authenticated: bool,
        #[serde(default)]
        is_admin: bool,
        user: Option<SessionUser>,
    }

    #[derive(Deserialize)]
    struct StarActionResponse {
        success: bool,
        starred: bool,
        error: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ExportPayload {
        exported_at: String,
        profile: Option<UserProfile>,
        modules: Vec<RegistryModule>,
        stars: Vec<RegistryModule>,
    }

    struct LoginResult {
        jar: CookieJar,
        redirect: String,
    }

    async fn login_with_redirect(app: &Router, redirect_to: Option<&str>) -> LoginResult {
        let mut jar = CookieJar::default();
        let uri = match redirect_to {
            Some(target) => format!("/auth/github?redirect_to={target}"),
            None => "/auth/github".to_string(),
        };
        let response = app
            .clone()
            .oneshot(build_request(
                Method::GET,
                &uri,
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await
            .expect("auth redirect");
        jar.update(&response);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("auth location")
            .to_str()
            .expect("auth location str");
        let state = Url::parse(location)
            .expect("authorize url")
            .query_pairs()
            .find_map(|(key, value)| (key == "state").then(|| value.to_string()))
            .expect("state param");

        let callback_uri = format!("/auth/github/callback?code=test-code&state={state}");
        let response = app
            .clone()
            .oneshot(build_request(
                Method::GET,
                &callback_uri,
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await
            .expect("auth callback");
        jar.update(&response);
        let redirect = response
            .headers()
            .get(header::LOCATION)
            .map(|value| value.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        LoginResult { jar, redirect }
    }

    async fn fetch_csrf_token(app: &Router, jar: &mut CookieJar) -> String {
        let response = app
            .clone()
            .oneshot(build_request(
                Method::GET,
                "/api/csrf-token",
                jar,
                Body::empty(),
                None,
                None,
            ))
            .await
            .expect("csrf token");
        jar.update(&response);
        let payload: CsrfTokenResponse = response_json(response).await;
        payload.token
    }

    async fn response_json<T: DeserializeOwned>(response: Response) -> T {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        serde_json::from_slice(&bytes).expect("json")
    }

    fn build_request(
        method: Method,
        uri: &str,
        jar: &CookieJar,
        body: Body,
        content_type: Option<&str>,
        csrf_token: Option<&str>,
    ) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(cookie) = jar.header_value() {
            builder = builder.header(header::COOKIE, cookie);
        }
        if let Some(content_type) = content_type {
            builder = builder.header(header::CONTENT_TYPE, content_type);
        }
        if let Some(csrf_token) = csrf_token {
            builder = builder.header("x-csrf-token", csrf_token);
        }
        builder.body(body).expect("request")
    }

    struct MultipartPart {
        name: String,
        filename: Option<String>,
        content_type: Option<String>,
        data: Vec<u8>,
    }

    impl MultipartPart {
        fn text(name: &str, value: &str) -> Self {
            Self {
                name: name.to_string(),
                filename: None,
                content_type: None,
                data: value.as_bytes().to_vec(),
            }
        }

        fn file(name: &str, filename: &str, content_type: &str, data: Vec<u8>) -> Self {
            Self {
                name: name.to_string(),
                filename: Some(filename.to_string()),
                content_type: Some(content_type.to_string()),
                data,
            }
        }
    }

    fn multipart_body(parts: Vec<MultipartPart>) -> (String, Vec<u8>) {
        let boundary = "boundary-test";
        let mut body = Vec::new();
        for part in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            if let Some(filename) = part.filename.as_deref() {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                        part.name, filename
                    )
                    .as_bytes(),
                );
                if let Some(content_type) = part.content_type.as_deref() {
                    body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
                }
                body.extend_from_slice(b"\r\n");
                body.extend_from_slice(&part.data);
                body.extend_from_slice(b"\r\n");
            } else {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                        part.name
                    )
                    .as_bytes(),
                );
                body.extend_from_slice(&part.data);
                body.extend_from_slice(b"\r\n");
            }
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    fn versioned<T>(payload: T) -> Versioned<T> {
        Versioned {
            version: 1,
            payload,
        }
    }

    fn sample_module(uuid: &str, name: &str) -> RegistryModule {
        RegistryModule {
            uuid: uuid.to_string(),
            name: name.to_string(),
            description: "Sample module".to_string(),
            author: "octo".to_string(),
            category: ModuleCategory::Weather,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/octo/sample".to_string(),
            downloads: 42,
            version: Some("1.0.0".to_string()),
            last_updated: Some(TEST_DATE.to_string()),
            rating: Some(4.2),
            verified_author: true,
            tags: vec!["sample".to_string()],
            checksum: Some("checksum".to_string()),
            license: Some("MIT".to_string()),
        }
    }

    fn sample_starred_module(uuid: &str, name: &str) -> StarredModule {
        StarredModule {
            module: sample_module(uuid, name),
            starred_at: TEST_DATE.to_string(),
        }
    }

    fn sample_collection(id: i64) -> Collection {
        Collection {
            id,
            user_id: 1,
            name: format!("Collection {id}"),
            description: Some("Test collection".to_string()),
            visibility: "public".to_string(),
            module_count: 1,
            owner: CollectionOwner {
                username: "octo".to_string(),
                display_name: Some("Octo".to_string()),
                avatar_url: Some("http://example.com/avatar.png".to_string()),
            },
            created_at: TEST_DATE.to_string(),
            updated_at: TEST_DATE.to_string(),
        }
    }

    fn sample_collection_detail(id: i64) -> CollectionDetailResponse {
        CollectionDetailResponse {
            collection: sample_collection(id),
            modules: vec![CollectionModule {
                uuid: "weather-wttr@octo".to_string(),
                name: "Weather".to_string(),
                author: "octo".to_string(),
                category: ModuleCategory::Weather,
                note: Some("Pinned".to_string()),
                position: 1,
                added_at: TEST_DATE.to_string(),
            }],
        }
    }

    fn sample_preferences() -> NotificationPreferences {
        NotificationPreferences {
            downloads_enabled: true,
            comments_enabled: false,
            stars_enabled: true,
            updates_enabled: true,
            announcements_enabled: false,
            email_downloads: false,
            email_comments: false,
            email_stars: true,
            email_updates: false,
            email_announcements: false,
        }
    }

    fn sample_admin_stats() -> AdminStats {
        AdminStats {
            total_modules: 10,
            total_users: 5,
            total_downloads: 200,
            pending_submissions: 2,
        }
    }

    fn sample_submission(id: i64) -> Submission {
        Submission {
            id,
            submitter_id: 1,
            uuid: "weather-wttr@octo".to_string(),
            name: "Weather".to_string(),
            description: "Weather module".to_string(),
            category: "weather".to_string(),
            version: "1.0.0".to_string(),
            repo_url: "https://github.com/octo/weather".to_string(),
            status: "pending".to_string(),
            rejection_reason: None,
            submitted_at: TEST_DATE.to_string(),
            reviewed_at: None,
            reviewed_by: None,
            submitter_username: "octo".to_string(),
        }
    }

    fn sample_user_profile(username: &str, role: UserRole) -> UserProfile {
        UserProfile {
            id: 1,
            username: username.to_string(),
            display_name: Some("Octo".to_string()),
            avatar_url: Some("http://example.com/avatar.png".to_string()),
            bio: None,
            website_url: None,
            github_url: Some("https://github.com/octo".to_string()),
            twitter_url: None,
            bluesky_url: None,
            discord_url: None,
            sponsor_url: None,
            verified_author: false,
            role,
            module_count: 2,
            created_at: TEST_DATE.to_string(),
        }
    }

    fn mock_router(state: MockState) -> Router {
        Router::new()
            .route("/login/oauth/access_token", post(mock_access_token))
            .route("/user", get(mock_github_user))
            .route("/user/emails", get(mock_github_emails))
            .route("/emails", post(mock_resend_send_email))
            .route("/api/v1/auth/sync", post(mock_auth_sync))
            .route("/api/v1/admin/stats", get(mock_admin_stats))
            .route("/api/v1/admin/submissions", get(mock_admin_submissions))
            .route(
                "/api/v1/admin/submissions/{id}/approve",
                post(mock_admin_approve),
            )
            .route(
                "/api/v1/admin/submissions/{id}/reject",
                post(mock_admin_reject),
            )
            .route("/api/v1/admin/users/{id}/verify", post(mock_admin_verify))
            .route(
                "/api/v1/collections",
                get(mock_collections).post(mock_collection_create),
            )
            .route(
                "/api/v1/collections/{id}",
                get(mock_collection_detail)
                    .patch(mock_collection_update)
                    .delete(mock_collection_delete),
            )
            .route(
                "/api/v1/collections/{id}/modules",
                post(mock_collection_add_module),
            )
            .route(
                "/api/v1/collections/{id}/modules/{uuid}",
                delete(mock_collection_remove_module),
            )
            .route(
                "/api/v1/users/me",
                get(mock_users_me)
                    .patch(mock_users_me_patch)
                    .delete(mock_users_me_delete),
            )
            .route("/api/v1/modules/mine", get(mock_modules_mine))
            .route("/api/v1/users/me/stars", get(mock_stars_index))
            .route("/api/v1/stars/sync", post(mock_stars_sync))
            .route(
                "/api/v1/modules/{uuid}/star",
                get(mock_star_status)
                    .post(mock_star_action)
                    .delete(mock_star_action),
            )
            .route(
                "/api/v1/notifications/preferences",
                get(mock_preferences_get).patch(mock_preferences_patch),
            )
            .route("/api/v1/modules", post(mock_module_create))
            .route(
                "/api/v1/modules/{uuid}/versions/{version}/upload",
                post(mock_module_upload),
            )
            .route(
                "/api/v1/modules/{uuid}/versions/{version}/publish",
                post(mock_module_publish),
            )
            .route(
                "/api/v1/modules/{uuid}/screenshots",
                post(mock_screenshot_upload),
            )
            .route(
                "/api/v1/modules/{uuid}/screenshots/{id}",
                delete(mock_screenshot_delete),
            )
            .route("/turnstile/v0/siteverify", post(mock_turnstile))
            .with_state(state)
    }

    async fn mock_access_token() -> Json<serde_json::Value> {
        Json(json!({
            "access_token": "token-123",
            "token_type": "bearer",
            "scope": "read:user user:email"
        }))
    }

    async fn mock_github_user(State(state): State<MockState>) -> Json<serde_json::Value> {
        Json(json!({
            "login": state.github_login,
            "name": "Octo",
            "avatar_url": "http://example.com/avatar.png"
        }))
    }

    async fn mock_github_emails(State(state): State<MockState>) -> Json<Vec<serde_json::Value>> {
        Json(vec![json!({
            "email": state.github_email,
            "primary": true,
            "verified": true
        })])
    }

    async fn mock_resend_send_email(
        State(state): State<MockState>,
        Json(payload): Json<ResendEmailRequest>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        state.record_export_email(payload);
        (StatusCode::OK, Json(json!({ "id": "email_123" })))
    }

    async fn mock_admin_stats() -> Json<Versioned<AdminStats>> {
        Json(versioned(sample_admin_stats()))
    }

    async fn mock_admin_submissions() -> Json<Versioned<SubmissionsResponse>> {
        Json(versioned(SubmissionsResponse {
            submissions: vec![sample_submission(1)],
            total: 1,
        }))
    }

    async fn mock_admin_approve() -> StatusCode {
        StatusCode::OK
    }

    async fn mock_admin_reject(Json(payload): Json<RejectRequest>) -> StatusCode {
        assert!(!payload.reason.is_empty());
        StatusCode::OK
    }

    async fn mock_admin_verify(Path(id): Path<i64>) -> Json<Versioned<VerifyResponse>> {
        Json(versioned(VerifyResponse {
            user_id: id,
            verified_author: true,
        }))
    }

    async fn mock_collections() -> Json<Versioned<CollectionsResponse>> {
        Json(versioned(CollectionsResponse {
            collections: vec![sample_collection(1)],
            total: 1,
        }))
    }

    async fn mock_collection_detail(
        Path(id): Path<i64>,
    ) -> Json<Versioned<CollectionDetailResponse>> {
        Json(versioned(sample_collection_detail(id)))
    }

    async fn mock_collection_create(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.get("name").is_some());
        StatusCode::OK
    }

    async fn mock_collection_update(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.as_object().is_some());
        StatusCode::OK
    }

    async fn mock_collection_delete() -> StatusCode {
        StatusCode::OK
    }

    async fn mock_collection_add_module(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.get("module_uuid").is_some());
        StatusCode::OK
    }

    async fn mock_collection_remove_module() -> StatusCode {
        StatusCode::OK
    }

    async fn mock_users_me(State(state): State<MockState>) -> Json<UserProfile> {
        Json(sample_user_profile(
            &state.github_login,
            state.profile_role.clone(),
        ))
    }

    async fn mock_auth_sync(State(state): State<MockState>) -> StatusCode {
        state.record_sync_call();
        StatusCode::OK
    }

    async fn mock_users_me_patch(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.as_object().is_some());
        StatusCode::OK
    }

    async fn mock_users_me_delete() -> StatusCode {
        StatusCode::OK
    }

    async fn mock_modules_mine() -> Json<Versioned<ModulesResponse>> {
        Json(versioned(ModulesResponse {
            modules: vec![sample_module("weather-wttr@octo", "Weather")],
            total: 1,
        }))
    }

    async fn mock_stars_index() -> Json<Versioned<StarsResponse>> {
        Json(versioned(StarsResponse {
            modules: vec![sample_starred_module("weather-wttr@octo", "Weather")],
            total: 1,
        }))
    }

    async fn mock_stars_sync(
        Json(payload): Json<SyncStarsRequest>,
    ) -> Json<Versioned<SyncStarsResponse>> {
        Json(versioned(SyncStarsResponse {
            synced: payload.uuids.len() as i64,
        }))
    }

    async fn mock_star_status() -> Json<Versioned<StarStatusResponse>> {
        Json(versioned(StarStatusResponse {
            starred: false,
            star_count: 3,
        }))
    }

    async fn mock_star_action() -> StatusCode {
        StatusCode::OK
    }

    async fn mock_preferences_get() -> Json<Versioned<NotificationPreferences>> {
        Json(versioned(sample_preferences()))
    }

    async fn mock_preferences_patch(
        Json(payload): Json<NotificationPreferences>,
    ) -> Json<Versioned<NotificationPreferences>> {
        Json(versioned(payload))
    }

    async fn mock_module_create(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.get("uuid").is_some());
        StatusCode::OK
    }

    async fn mock_module_upload(bytes: Bytes) -> StatusCode {
        assert!(!bytes.is_empty());
        StatusCode::OK
    }

    async fn mock_module_publish(Json(payload): Json<serde_json::Value>) -> StatusCode {
        assert!(payload.as_object().is_some());
        StatusCode::OK
    }

    async fn mock_screenshot_upload(bytes: Bytes) -> Json<ScreenshotUploadResponse> {
        assert!(!bytes.is_empty());
        Json(ScreenshotUploadResponse {
            version: 2,
            data: ScreenshotUploadData {
                id: 12,
                r2_key: "modules/weather.png".to_string(),
                url: "https://cdn.example.com/weather.png".to_string(),
            },
        })
    }

    async fn mock_screenshot_delete() -> Json<ScreenshotDeleteResponse> {
        Json(ScreenshotDeleteResponse {
            version: 3,
            data: ScreenshotDeleteData { deleted: true },
        })
    }

    async fn mock_turnstile() -> Json<serde_json::Value> {
        Json(json!({"success": true}))
    }

    #[tokio::test]
    async fn github_auth_redirects_to_authorize_url() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/auth/github",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location str");
        let url = Url::parse(location).expect("location url");
        assert_eq!(url.path(), "/login/oauth/authorize");
        let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("client_id"), Some(&TEST_CLIENT_ID.to_string()));
        assert_eq!(
            params.get("redirect_uri"),
            Some(&format!("{TEST_APP_BASE_URL}/auth/github/callback"))
        );
        let scope = params.get("scope").cloned().unwrap_or_default();
        assert!(scope.contains("read:user"));
        assert!(scope.contains("user:email"));
        assert!(params.contains_key("state"));
    }

    #[tokio::test]
    async fn github_callback_rejects_invalid_state() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/auth/github/callback?code=test&state=invalid",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn github_callback_redirects_to_requested_target() {
        let harness = TestHarness::new().await;
        let result = login_with_redirect(&harness.app, Some("/dashboard")).await;

        assert_eq!(result.redirect, "/dashboard");
    }

    #[tokio::test]
    async fn session_status_unauthenticated() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers().clone();
        let payload: SessionResponse = response_json(response).await;
        assert!(!payload.authenticated);
        assert!(payload.user.is_none());
        let cookies: Vec<_> = headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .collect();
        assert!(cookies.iter().any(|value| value.contains("profile_cache=")));
    }

    #[tokio::test]
    async fn session_status_authenticated() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        let payload: SessionResponse = response_json(response).await;
        assert!(payload.authenticated);
        assert!(!payload.is_admin);
        let user = payload.user.expect("user");
        assert_eq!(user.login, "octo");
        assert_eq!(user.email.as_deref(), Some("octo@example.com"));
    }

    #[tokio::test]
    async fn session_status_admin() {
        let harness = TestHarness::new_with_user("admin").await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        let payload: SessionResponse = response_json(response).await;
        assert!(payload.authenticated);
        assert!(payload.is_admin);
        let user = payload.user.expect("user");
        assert_eq!(user.login, "admin");
        assert_eq!(user.email.as_deref(), Some("admin@example.com"));
    }

    #[tokio::test]
    async fn session_status_sets_admin_from_profile_role() {
        let harness = TestHarness::new_with_role("octo", UserRole::Admin).await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        let payload: SessionResponse = response_json(response).await;
        assert!(payload.authenticated);
        assert!(payload.is_admin);
    }

    #[tokio::test]
    async fn session_status_syncs_user() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(harness.sync_calls(), 1);
    }

    #[tokio::test]
    async fn logout_clears_session() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let response = harness
            .request(build_request(
                Method::POST,
                "/auth/logout",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        jar.update(&response);

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/session",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: SessionResponse = response_json(response).await;
        assert!(!payload.authenticated);
    }

    #[tokio::test]
    async fn csrf_token_returns_value() {
        let harness = TestHarness::new().await;
        let mut jar = CookieJar::default();
        let token = fetch_csrf_token(&harness.app, &mut jar).await;

        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn protected_route_redirects_to_login() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/dashboard",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location")
            .to_str()
            .expect("location str");
        assert_eq!(location, "/login?redirect_to=/dashboard");
    }

    #[tokio::test]
    async fn settings_route_redirects_to_login() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/settings/profile",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location")
            .to_str()
            .expect("location str");
        assert_eq!(location, "/login?redirect_to=/settings/profile");
    }

    #[tokio::test]
    async fn protected_route_allows_authenticated() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/dashboard",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn admin_route_forbidden_for_non_admin() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/admin",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_route_allows_admin() {
        let harness = TestHarness::new_with_user("admin").await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/admin",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn admin_route_allows_admin_role() {
        let harness = TestHarness::new_with_role("octo", UserRole::Moderator).await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/admin",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn admin_stats_requires_admin() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/admin/stats",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/admin/stats",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_stats_returns_payload_for_admin() {
        let harness = TestHarness::new_with_user("admin").await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/admin/stats",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        let payload: Versioned<AdminStats> = response_json(response).await;
        assert_eq!(payload.version, 1);
        assert_eq!(payload.payload.total_modules, 10);
    }

    #[tokio::test]
    async fn admin_submissions_endpoints_work() {
        let harness = TestHarness::new_with_user("admin").await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let response = harness
            .request(build_request(
                Method::GET,
                "/api/admin/submissions",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<SubmissionsResponse> = response_json(response).await;
        assert_eq!(payload.payload.total, 1);

        let response = harness
            .request(build_request(
                Method::POST,
                "/api/admin/submissions/1/approve",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let reject_body = serde_json::to_vec(&json!({"reason": "Nope"})).expect("json");
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/admin/submissions/1/reject",
                &jar,
                Body::from(reject_body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = harness
            .request(build_request(
                Method::POST,
                "/api/admin/users/7/verify",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        let payload: Versioned<VerifyResponse> = response_json(response).await;
        assert_eq!(payload.payload.user_id, 7);
    }

    #[tokio::test]
    async fn collections_endpoints_require_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/collections",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn collections_index_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/collections",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<CollectionsResponse> = response_json(response).await;
        assert_eq!(payload.payload.total, 1);
    }

    #[tokio::test]
    async fn collection_detail_allows_unauthenticated() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/collections/1",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<CollectionDetailResponse> = response_json(response).await;
        assert_eq!(payload.payload.collection.id, 1);
    }

    #[tokio::test]
    async fn collection_mutations_require_csrf() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/collections",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn collection_mutations_succeed_with_csrf() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let create_body = serde_json::to_vec(&json!({
            "name": "Favorites",
            "description": "Best modules",
            "visibility": "public"
        }))
        .expect("json");
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/collections",
                &jar,
                Body::from(create_body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let update_body = serde_json::to_vec(&json!({"name": "Updated"})).expect("json");
        let response = harness
            .request(build_request(
                Method::PATCH,
                "/api/collections/1",
                &jar,
                Body::from(update_body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let add_body =
            serde_json::to_vec(&json!({"module_uuid": "weather-wttr@octo"})).expect("json");
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/collections/1/modules",
                &jar,
                Body::from(add_body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = harness
            .request(build_request(
                Method::DELETE,
                "/api/collections/1/modules/weather-wttr@octo",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = harness
            .request(build_request(
                Method::DELETE,
                "/api/collections/1",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn notification_preferences_requires_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/notifications/preferences",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn notification_preferences_patch_updates() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let payload = sample_preferences();
        let body = serde_json::to_vec(&payload).expect("json");
        let response = harness
            .request(build_request(
                Method::PATCH,
                "/api/notifications/preferences",
                &jar,
                Body::from(body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        let payload: Versioned<NotificationPreferences> = response_json(response).await;
        assert!(payload.payload.stars_enabled);
    }

    #[tokio::test]
    async fn stars_index_unauth_returns_empty() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/stars",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<StarsResponse> = response_json(response).await;
        assert_eq!(payload.version, 0);
        assert_eq!(payload.payload.total, 0);
    }

    #[tokio::test]
    async fn stars_index_auth_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/stars",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<StarsResponse> = response_json(response).await;
        assert_eq!(payload.payload.total, 1);
    }

    #[tokio::test]
    async fn stars_sync_requires_auth() {
        let harness = TestHarness::new().await;
        let body = serde_json::to_vec(&SyncStarsRequest {
            uuids: vec!["weather-wttr@octo".to_string()],
            is_public: Some(true),
        })
        .expect("json");
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/stars/sync",
                &CookieJar::default(),
                Body::from(body),
                Some("application/json"),
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn stars_sync_with_csrf() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let body = serde_json::to_vec(&SyncStarsRequest {
            uuids: vec!["weather-wttr@octo".to_string()],
            is_public: Some(true),
        })
        .expect("json");
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/stars/sync",
                &jar,
                Body::from(body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        let payload: Versioned<SyncStarsResponse> = response_json(response).await;
        assert_eq!(payload.payload.synced, 1);
    }

    #[tokio::test]
    async fn star_status_is_public() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/modules/weather-wttr@octo/star",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        let payload: Versioned<StarStatusResponse> = response_json(response).await;
        assert_eq!(payload.payload.star_count, 3);
    }

    #[tokio::test]
    async fn star_actions_require_auth_and_csrf() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let response = harness
            .request(build_request(
                Method::POST,
                "/api/modules/weather-wttr@octo/star",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        let payload: StarActionResponse = response_json(response).await;
        assert!(payload.success);
        assert!(payload.starred);
        assert!(payload.error.is_none());

        let response = harness
            .request(build_request(
                Method::DELETE,
                "/api/modules/weather-wttr@octo/star",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        let payload: StarActionResponse = response_json(response).await;
        assert!(payload.success);
        assert!(!payload.starred);
        assert!(payload.error.is_none());
    }

    #[tokio::test]
    async fn profile_update_and_delete_require_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::PATCH,
                "/api/users/me",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn profile_update_and_delete_succeed_with_csrf() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let update_body = serde_json::to_vec(&json!({
            "display_name": "Octo",
            "bio": "Hello"
        }))
        .expect("json");
        let response = harness
            .request(build_request(
                Method::PATCH,
                "/api/users/me",
                &jar,
                Body::from(update_body),
                Some("application/json"),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = harness
            .request(build_request(
                Method::DELETE,
                "/api/users/me",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn profile_get_requires_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/users/me",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn profile_get_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/users/me",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: UserProfile = response_json(response).await;
        assert_eq!(payload.username, "octo");
    }

    #[tokio::test]
    async fn modules_mine_requires_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/modules/mine",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn modules_mine_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/modules/mine",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Versioned<ModulesResponse> = response_json(response).await;
        assert_eq!(payload.payload.modules.len(), 1);
        assert_eq!(payload.payload.total, 1);
    }

    #[tokio::test]
    async fn export_data_requires_auth() {
        let harness = TestHarness::new().await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/users/me/export",
                &CookieJar::default(),
                Body::empty(),
                None,
                None,
            ))
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn export_data_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/users/me/export",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers().clone();
        let payload: ExportPayload = response_json(response).await;
        assert!(!payload.exported_at.is_empty());
        assert!(payload.exported_at.contains('T'));
        assert_eq!(
            payload.profile.map(|profile| profile.username),
            Some("octo".to_string())
        );
        assert_eq!(payload.modules.len(), 1);
        assert_eq!(payload.stars.len(), 1);

        let disposition = headers
            .get(header::CONTENT_DISPOSITION)
            .expect("disposition")
            .to_str()
            .expect("disposition str");
        assert!(disposition.contains("attachment"));
        assert!(disposition.contains("barforge-export-"));
    }

    #[tokio::test]
    async fn export_data_sends_confirmation_email() {
        let harness = TestHarness::new().await;
        let LoginResult { jar, .. } = login_with_redirect(&harness.app, None).await;
        let response = harness
            .request(build_request(
                Method::GET,
                "/api/users/me/export",
                &jar,
                Body::empty(),
                None,
                None,
            ))
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let emails = harness.export_emails();
        assert_eq!(emails.len(), 1);
        let email = &emails[0];
        assert_eq!(email.from, "Barforge <noreply@barforge.dev>".to_string());
        assert_eq!(email.to, vec!["octo@example.com".to_string()]);
        assert!(email.subject.contains("Barforge"));
        assert!(email.text.contains("export"));
    }

    #[tokio::test]
    async fn screenshot_upload_rejects_missing_file() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![MultipartPart::text("alt_text", "Nice")]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/modules/weather-wttr@octo/screenshots",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn screenshot_upload_rejects_invalid_type() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![MultipartPart::file(
            "screenshot",
            "shot.txt",
            "text/plain",
            b"not an image".to_vec(),
        )]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/modules/weather-wttr@octo/screenshots",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn screenshot_upload_accepts_valid_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::file(
                "screenshot",
                "shot.png",
                "image/png",
                b"image-bytes".to_vec(),
            ),
            MultipartPart::text("alt_text", "Weather view"),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/modules/weather-wttr@octo/screenshots",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        let payload: ScreenshotUploadResponse = response_json(response).await;
        assert_eq!(payload.data.id, 12);
    }

    #[tokio::test]
    async fn screenshot_delete_returns_payload() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let response = harness
            .request(build_request(
                Method::DELETE,
                "/api/modules/weather-wttr@octo/screenshots/12",
                &jar,
                Body::empty(),
                None,
                Some(&csrf_token),
            ))
            .await;
        let payload: ScreenshotDeleteResponse = response_json(response).await;
        assert!(payload.data.deleted);
    }

    #[tokio::test]
    async fn upload_module_rejects_missing_package() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::text("name", "Weather"),
            MultipartPart::text("description", "Weather module"),
            MultipartPart::text("category", "weather"),
            MultipartPart::text("version", "1.2.3"),
            MultipartPart::text("license", "MIT"),
            MultipartPart::text("repo_url", "https://github.com/octo/weather"),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/upload",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_module_rejects_invalid_extension() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::text("name", "Weather"),
            MultipartPart::text("description", "Weather module"),
            MultipartPart::text("category", "weather"),
            MultipartPart::text("version", "1.2.3"),
            MultipartPart::text("license", "MIT"),
            MultipartPart::text("repo_url", "https://github.com/octo/weather"),
            MultipartPart::file("package", "module.zip", "application/zip", vec![1, 2, 3]),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/upload",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_module_rejects_invalid_version() {
        let harness = TestHarness::new().await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::text("name", "Weather"),
            MultipartPart::text("description", "Weather module"),
            MultipartPart::text("category", "weather"),
            MultipartPart::text("version", "1.2"),
            MultipartPart::text("license", "MIT"),
            MultipartPart::text("repo_url", "https://github.com/octo/weather"),
            MultipartPart::file(
                "package",
                "module.tar.gz",
                "application/gzip",
                vec![1, 2, 3],
            ),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/upload",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_module_requires_turnstile_when_secret_set() {
        let harness = TestHarness::new_with_turnstile("octo").await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::text("name", "Weather"),
            MultipartPart::text("description", "Weather module"),
            MultipartPart::text("category", "weather"),
            MultipartPart::text("version", "1.2.3"),
            MultipartPart::text("license", "MIT"),
            MultipartPart::text("repo_url", "https://github.com/octo/weather"),
            MultipartPart::file(
                "package",
                "module.tar.gz",
                "application/gzip",
                vec![1, 2, 3],
            ),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/upload",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_module_accepts_valid_payload() {
        let harness = TestHarness::new_with_turnstile("octo").await;
        let LoginResult { mut jar, .. } = login_with_redirect(&harness.app, None).await;
        let csrf_token = fetch_csrf_token(&harness.app, &mut jar).await;

        let (content_type, body) = multipart_body(vec![
            MultipartPart::text("name", "Weather"),
            MultipartPart::text("description", "Weather module"),
            MultipartPart::text("category", "weather"),
            MultipartPart::text("version", "1.2.3"),
            MultipartPart::text("license", "MIT"),
            MultipartPart::text("repo_url", "https://github.com/octo/weather"),
            MultipartPart::text("cf-turnstile-response", "turnstile-token"),
            MultipartPart::file(
                "package",
                "module.tar.gz",
                "application/gzip",
                vec![1, 2, 3],
            ),
        ]);
        let response = harness
            .request(build_request(
                Method::POST,
                "/api/upload",
                &jar,
                Body::from(body),
                Some(&content_type),
                Some(&csrf_token),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
