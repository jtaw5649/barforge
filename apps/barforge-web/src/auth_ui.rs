use crate::Route;
use crate::api;
use crate::auth_redirect;
use crate::state::{RemoteState, use_session_state};
use dioxus::prelude::*;
use dioxus_router::use_route;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AuthStatus {
    pub(crate) state: RemoteState<api::SessionResponse>,
    pub(crate) authenticated: bool,
    pub(crate) is_admin: bool,
}

pub(crate) fn use_auth() -> AuthStatus {
    let state = use_session_state();
    let (authenticated, is_admin) = match state {
        RemoteState::Ready(ref session) => (session.authenticated, session.is_admin),
        _ => (false, false),
    };

    AuthStatus {
        state,
        authenticated,
        is_admin,
    }
}

pub(crate) fn github_auth_href(redirect_to: Option<&str>) -> String {
    auth_redirect::sanitize_redirect_target(redirect_to)
        .map(auth_redirect::encode_query_value)
        .map(|encoded| format!("/auth/github?redirect_to={encoded}"))
        .unwrap_or_else(|| "/auth/github".to_string())
}

pub(crate) fn login_redirect_target(
    authenticated: bool,
    redirect_to: Option<&str>,
) -> Option<String> {
    if !authenticated {
        return None;
    }

    let target = auth_redirect::sanitize_redirect_target(redirect_to).unwrap_or("/");

    Some(target.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthGateKind {
    User,
    Admin,
}

pub(crate) fn auth_gate_view(
    state: &RemoteState<api::SessionResponse>,
    gate: AuthGateKind,
    redirect_to: &str,
    children: Element,
) -> Element {
    let auth_href = github_auth_href(Some(redirect_to));
    let login_card = |title: &str, message: &str| {
        rsx! {
            section { class: "login-route auth-gate",
                div { class: "login-card",
                    h1 { "{title}" }
                    p { "{message}" }
                    a { class: "github-btn", href: "{auth_href}",
                        svg {
                            class: "github-icon",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            path {
                                d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                            }
                        }
                        "Continue with GitHub"
                    }
                }
            }
        }
    };

    match state {
        RemoteState::Ready(session) => {
            if session.authenticated {
                if gate == AuthGateKind::Admin && !session.is_admin {
                    rsx! {
                        section { class: "login-route auth-gate",
                            div { class: "login-card",
                                h1 { "Admin access required" }
                                p { "You do not have permission to view this page." }
                                a { class: "ghost-button", href: "/modules", "Back to Modules" }
                            }
                        }
                    }
                } else {
                    children
                }
            } else {
                login_card(
                    "Log in to continue",
                    "This page is available to registered members.",
                )
            }
        }
        RemoteState::Loading => login_card("Checking your session", "Verifying your access..."),
        RemoteState::Error(_) => login_card(
            "Unable to verify session",
            "Please sign in again to continue.",
        ),
        RemoteState::Unavailable => children,
    }
}

#[component]
pub(crate) fn ProtectedRoute(children: Element) -> Element {
    let auth = use_auth();
    let route: Route = use_route();
    let redirect_to = route.to_string();

    auth_gate_view(&auth.state, AuthGateKind::User, &redirect_to, children)
}

#[component]
pub(crate) fn AdminGate(children: Element) -> Element {
    let auth = use_auth();
    let route: Route = use_route();
    let redirect_to = route.to_string();

    auth_gate_view(&auth.state, AuthGateKind::Admin, &redirect_to, children)
}

#[cfg(test)]
mod tests {
    use super::login_redirect_target;

    #[test]
    fn login_redirect_target_requires_authentication() {
        assert_eq!(login_redirect_target(false, Some("/dashboard")), None);
    }

    #[test]
    fn login_redirect_target_prefers_safe_target() {
        assert_eq!(
            login_redirect_target(true, Some("/dashboard")),
            Some("/dashboard".to_string())
        );
    }

    #[test]
    fn login_redirect_target_falls_back_to_root_on_unsafe_target() {
        assert_eq!(
            login_redirect_target(true, Some("https://evil.com")),
            Some("/".to_string())
        );
    }
}
