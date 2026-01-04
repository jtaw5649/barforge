use dioxus::{document, prelude::*};
use dioxus_fullstack::{FullstackContext, StatusCode};
use dioxus_history::history;
use dioxus_router::Router;
use keyboard_types::{Key, Modifiers};
use manganis::{Asset, asset};
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::KeyboardEvent as WebKeyboardEvent;

use crate::Route;
use crate::auth_redirect;
use crate::auth_ui::use_auth;
use crate::command_palette::{
    PaletteAction, PaletteItem, PaletteItemKind, PaletteMode, PaletteState, filter_palette,
    palette_items,
};
use crate::notifications::{NotificationStatus, use_notifications, use_notifications_provider};
use crate::recently_viewed::use_recently_viewed_provider;
use crate::stars::use_stars_provider;
use crate::state::{RemoteState, SessionStateProvider};
use crate::theme::{ThemePreference, use_theme};

pub(crate) fn app_error_fallback(errors: ErrorContext) -> Element {
    let error = errors.error();
    let http_error = error.clone().map(FullstackContext::commit_error_status);
    let status = http_error
        .as_ref()
        .map(|value| value.status)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let title = match status {
        StatusCode::NOT_FOUND => "Page not found",
        _ => "Something went wrong",
    };
    let detail = match status {
        StatusCode::NOT_FOUND => "We couldn’t find that page.".to_string(),
        _ => "Please try again or return later.".to_string(),
    };

    rsx! {
        section { class: "error-boundary",
            h1 { "{title}" }
            p { class: "error-detail", "{detail}" }
            a { class: "ghost-button", href: "/", "Back to Home" }
        }
    }
}

#[component]
pub fn App() -> Element {
    static MAIN_CSS: Asset = asset!("/assets/main.css");
    static FAVICON: Asset = asset!("/assets/favicon.ico");
    static APPLE_TOUCH_ICON: Asset = asset!("/assets/apple-touch-icon.png");

    let head_links = if cfg!(target_arch = "wasm32") {
        rsx! {
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
            document::Link { rel: "prefetch", href: "/modules" }
            document::Link { rel: "prefetch", href: "/upload" }
        }
    } else {
        rsx! {}
    };

    let ssr_links = if cfg!(target_arch = "wasm32") {
        rsx! {}
    } else {
        rsx! {
            link { rel: "icon", href: FAVICON }
            link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
            link { rel: "prefetch", href: "/modules" }
            link { rel: "prefetch", href: "/upload" }
        }
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        {head_links}
        {ssr_links}
        SessionStateProvider {
            AppShell {
                ErrorBoundary { handle_error: app_error_fallback,
                    Router::<Route> {}
                }
            }
        }
    }
}

#[component]
pub fn AppEntry() -> Element {
    rsx!(App {})
}

#[component]
fn AppShell(children: Element) -> Element {
    let stars = use_stars_provider();
    let _recently_viewed = use_recently_viewed_provider();
    let notifications = use_notifications_provider();
    let auth = use_auth();
    let is_authenticated = auth.authenticated;
    let route_path = history().current_route();
    let login_href = auth_redirect::sanitize_redirect_target(Some(route_path.as_str()))
        .filter(|target| *target != "/")
        .map(auth_redirect::login_redirect_url)
        .unwrap_or_else(|| "/login".to_string());
    let user_login = match &auth.state {
        RemoteState::Ready(session) => session
            .user
            .as_ref()
            .map(|user| user.login.clone())
            .unwrap_or_else(|| "user".to_string()),
        _ => "user".to_string(),
    };
    let user_handle = format!("@{user_login}");
    let user_profile_href = format!("/users/{user_login}");
    let user_initial = user_login.chars().next().unwrap_or('U');
    let mobile_menu_open = use_signal(|| false);
    let mobile_menu_is_open = *mobile_menu_open.read();
    let mobile_menu_label = if mobile_menu_is_open {
        "Close menu"
    } else {
        "Open menu"
    };
    let mobile_menu_class = if mobile_menu_is_open {
        "mobile-menu open"
    } else {
        "mobile-menu"
    };
    let mobile_backdrop_class = if mobile_menu_is_open {
        "mobile-menu-backdrop open"
    } else {
        "mobile-menu-backdrop"
    };
    let mobile_menu_expanded = if mobile_menu_is_open { "true" } else { "false" };
    let mobile_menu_hidden = if mobile_menu_is_open { "false" } else { "true" };
    let user_menu_open = use_signal(|| false);
    let user_menu_is_open = *user_menu_open.read();
    let user_menu_class = if user_menu_is_open {
        "dropdown-menu open"
    } else {
        "dropdown-menu"
    };
    let user_menu_expanded = if user_menu_is_open { "true" } else { "false" };
    let user_menu_hidden = if user_menu_is_open { "false" } else { "true" };
    let toggle_mobile_menu = {
        let mut mobile_menu_open = mobile_menu_open;
        move |_| {
            let next = !*mobile_menu_open.read();
            mobile_menu_open.set(next);
        }
    };
    let close_mobile_menu = {
        let mut mobile_menu_open = mobile_menu_open;
        move |_| {
            if *mobile_menu_open.read() {
                mobile_menu_open.set(false);
            }
        }
    };
    let toggle_user_menu = {
        let mut user_menu_open = user_menu_open;
        move |_| {
            let next = !*user_menu_open.read();
            user_menu_open.set(next);
        }
    };
    let close_user_menu = {
        let mut user_menu_open = user_menu_open;
        move |_| {
            if *user_menu_open.read() {
                user_menu_open.set(false);
            }
        }
    };

    let theme = use_theme();
    let theme_current = theme.current();
    let theme_label = theme_label(theme_current);
    let theme_aria = format!("Toggle theme (currently {theme_label})");
    let theme_title = format!("Theme: {theme_label}");
    let theme_icon = match theme_current {
        ThemePreference::Light => rsx!(svg {
            class: "theme-toggle-icon",
            width: "18",
            height: "18",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2" }
            path { d: "M12 20v2" }
            path { d: "m4.93 4.93 1.41 1.41" }
            path { d: "m17.66 17.66 1.41 1.41" }
            path { d: "M2 12h2" }
            path { d: "M20 12h2" }
            path { d: "m6.34 17.66-1.41 1.41" }
            path { d: "m19.07 4.93-1.41 1.41" }
        }),
        ThemePreference::Dark => rsx!(svg {
            class: "theme-toggle-icon",
            width: "18",
            height: "18",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401" }
        }),
        ThemePreference::System => rsx!(svg {
            class: "theme-toggle-icon",
            width: "18",
            height: "18",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { width: "20", height: "14", x: "2", y: "3", rx: "2" }
            line { x1: "8", x2: "16", y1: "21", y2: "21" }
            line { x1: "12", x2: "12", y1: "17", y2: "21" }
        }),
    };

    let palette_open_sig = use_signal(|| false);
    let palette_mode_sig = use_signal(|| PaletteMode::All);
    let palette_query_sig = use_signal(String::new);
    let palette = PaletteState::from_signals(palette_open_sig, palette_mode_sig, palette_query_sig);
    let palette_items = palette_items(is_authenticated, auth.is_admin);
    let palette_query = palette.query();
    let palette_mode = palette.mode();
    let palette_results = filter_palette(&palette_items, &palette_query, palette_mode);
    let palette_results_len = palette_results.len();
    let palette_has_results = palette_results_len > 0;
    let palette_selected = use_signal(|| 0usize);
    let palette_active_index =
        (*palette_selected.read()).min(palette_results_len.saturating_sub(1));
    let palette_is_open = *palette_open_sig.read();
    let palette_backdrop_class = if palette_is_open {
        "palette-backdrop open"
    } else {
        "palette-backdrop"
    };
    let palette_backdrop_hidden = if palette_is_open { "false" } else { "true" };

    let open_palette = {
        let mut sig = palette_open_sig;
        move |_| sig.set(true)
    };
    let open_palette_mobile = {
        let mut sig = palette_open_sig;
        move |_| sig.set(true)
    };
    let close_palette = {
        let palette = palette.clone();
        move |_| palette.close()
    };

    use_effect({
        let palette = palette.clone();
        let mut palette_selected = palette_selected;
        move || {
            let _ = palette.query();
            let _ = palette.mode();
            palette_selected.set(0);
        }
    });

    use_effect({
        let palette = palette.clone();
        let mut palette_selected = palette_selected;
        move || {
            let open = palette.is_open();
            if open {
                palette_selected.set(0);
                palette.set_query(String::new());
            }
            set_body_scroll_locked(open);
        }
    });

    #[cfg(target_arch = "wasm32")]
    use_effect({
        let palette = palette.clone();
        move || {
            let Some(window) = web_sys::window() else {
                return;
            };
            let palette = palette.clone();
            let handler =
                Closure::<dyn FnMut(WebKeyboardEvent)>::new(move |event: WebKeyboardEvent| {
                    let has_modifier = event.ctrl_key() || event.meta_key();
                    if !has_modifier || !event.shift_key() {
                        return;
                    }
                    if event.key().eq_ignore_ascii_case("k") {
                        event.prevent_default();
                        palette.open();
                    }
                });
            let _ = window
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
            handler.forget();
        }
    });

    use_effect({
        let stars = stars.clone();
        let notifications = notifications.clone();
        move || {
            stars.set_authenticated(is_authenticated);
            notifications.set_authenticated(is_authenticated);
            if is_authenticated {
                notifications.connect_stream();
            } else {
                notifications.disconnect_stream();
            }
        }
    });

    let toggle_theme = {
        let theme = theme.clone();
        move |_| theme.cycle()
    };
    let execute_palette_item = Rc::new({
        let palette = palette.clone();
        move |item: PaletteItem| {
            palette.close();
            if let Some(action) = item.action {
                execute_palette_action(action);
            }
            if let Some(path) = item.path {
                if path.starts_with("http") {
                    open_external_url(path);
                } else {
                    history().push(path.to_string());
                }
            }
        }
    });
    let handle_palette_keydown = {
        let palette = palette.clone();
        let execute_palette_item = execute_palette_item.clone();
        let mut palette_selected = palette_selected;
        let palette_results = palette_results.clone();
        move |event: KeyboardEvent| {
            let key = event.key();
            if key == Key::Escape {
                event.prevent_default();
                palette.close();
                return;
            }
            if key == Key::ArrowDown {
                event.prevent_default();
                let current = *palette_selected.read();
                let next = if palette_results.is_empty() {
                    0
                } else {
                    (current + 1).min(palette_results.len() - 1)
                };
                palette_selected.set(next);
                return;
            }
            if key == Key::ArrowUp {
                event.prevent_default();
                let current = *palette_selected.read();
                let next = current.saturating_sub(1);
                palette_selected.set(next);
                return;
            }
            if key == Key::Enter {
                if let Some(result) = palette_results.get(*palette_selected.read()) {
                    event.prevent_default();
                    execute_palette_item(result.item.clone());
                }
                return;
            }
            if key == Key::Tab && event.modifiers().contains(Modifiers::ALT) {
                event.prevent_default();
                let next = match palette.mode() {
                    PaletteMode::All => PaletteMode::Pages,
                    PaletteMode::Pages => PaletteMode::Commands,
                    PaletteMode::Commands => PaletteMode::All,
                };
                palette.set_mode(next);
            }
        }
    };

    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to main content" }
        div { class: "app-shell", tabindex: "-1",
            header { class: "site-header",
                div { class: "header-container",
                    a { class: "logo", href: "/",
                        svg {
                            class: "logo-mark",
                            width: "28",
                            height: "28",
                            view_box: "0 0 100 100",
                            fill: "none",
                            defs {
                                linearGradient { id: "logoGradHeader", x1: "0%", y1: "0%", x2: "0%", y2: "100%",
                                    stop { offset: "0%", stop_color: "#cba6f7" }
                                    stop { offset: "100%", stop_color: "#89b4fa" }
                                }
                            }
                            path { d: "M10 20 L90 20 L78 38 L22 38 Z", fill: "url(#logoGradHeader)" }
                            path { d: "M30 43 L70 43 L70 60 L30 60 Z", fill: "url(#logoGradHeader)", fill_opacity: "0.9" }
                            path { d: "M22 65 L78 65 L90 85 L10 85 Z", fill: "url(#logoGradHeader)", fill_opacity: "0.8" }
                        }
                        span { class: "logo-text",
                            span { class: "logo-title", "Barforge" }
                        }
                    }
                    nav { class: "nav-links", aria_label: "Main navigation",
                        a { href: "/modules", "Modules" }
                        {is_authenticated.then(|| rsx!(a { href: "/dashboard", "Dashboard" }))}
                    }
                    div { class: "header-actions",
                        div { class: "header-search",
                            button {
                                class: "search-trigger search-sm",
                                r#type: "button",
                                "aria-label": "Open search",
                                onclick: open_palette,
                                div { class: "search-icon",
                                    svg {
                                        class: "header-search-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        "aria-hidden": "true",
                                        path { d: "m21 21-4.34-4.34" }
                                        circle { cx: "11", cy: "11", r: "8" }
                                    }
                                }
                                span { class: "search-placeholder", "Search modules..." }
                                div { class: "search-shortcut",
                                    div { class: "kbd-group",
                                        kbd { class: "kbd", "Ctrl" }
                                        kbd { class: "kbd", "⇧" }
                                        kbd { class: "kbd", "K" }
                                    }
                                }
                            }
                        }
                        button {
                            class: "theme-toggle",
                            r#type: "button",
                            "aria-label": "{theme_aria}",
                            title: "{theme_title}",
                            onclick: toggle_theme,
                            div { class: "icon-wrapper", key: "{theme_label}",
                                {theme_icon}
                            }
                        }
                        {if is_authenticated {
                            rsx! {
                                NotificationCenter {}
                                div { class: "avatar-dropdown",
                                    button {
                                        class: "avatar-trigger",
                                        r#type: "button",
                                        "aria-label": "Open user menu",
                                        aria_expanded: "{user_menu_expanded}",
                                        aria_haspopup: "true",
                                        onclick: toggle_user_menu,
                                        span { class: "avatar-initial", "{user_initial}" }
                                    }
                                    div { class: "{user_menu_class}", role: "menu", aria_hidden: "{user_menu_hidden}",
                                        div { class: "dropdown-header",
                                            span { class: "avatar-initial avatar-initial-lg", "{user_initial}" }
                                            div { class: "dropdown-header-text",
                                                span { class: "dropdown-name", "{user_login}" }
                                                span { class: "dropdown-username", "{user_handle}" }
                                            }
                                        }
                                        div { class: "dropdown-section",
                                            a {
                                                class: "dropdown-link",
                                                href: "{user_profile_href}",
                                                role: "menuitem",
                                                onclick: close_user_menu,
                                                "Your profile"
                                            }
                                            a {
                                                class: "dropdown-link",
                                                href: "/dashboard",
                                                role: "menuitem",
                                                onclick: close_user_menu,
                                                "Your modules"
                                            }
                                            a {
                                                class: "dropdown-link",
                                                href: "/stars",
                                                role: "menuitem",
                                                onclick: close_user_menu,
                                                "Your stars"
                                            }
                                        }
                                        div { class: "dropdown-section",
                                            a {
                                                class: "dropdown-link",
                                                href: "/upload",
                                                role: "menuitem",
                                                onclick: close_user_menu,
                                                "Upload module"
                                            }
                                        }
                                        div { class: "dropdown-section",
                                            a {
                                                class: "dropdown-link",
                                                href: "/settings/profile",
                                                role: "menuitem",
                                                onclick: close_user_menu,
                                                "Settings"
                                            }
                                        }
                                        div { class: "dropdown-section dropdown-footer",
                                            form { action: "/auth/logout", method: "post",
                                                button { class: "dropdown-link", r#type: "submit", "Log out" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx!(a { class: "header-btn header-btn-primary", href: "{login_href}", "Log In" })
                        }}
                    }
                    div { class: "mobile-controls",
                        button {
                            class: "mobile-search",
                            r#type: "button",
                            "aria-label": "Open command palette",
                            onclick: open_palette_mobile,
                            svg {
                                class: "mobile-search-icon",
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "m21 21-4.34-4.34" }
                                circle { cx: "11", cy: "11", r: "8" }
                            }
                        }
                        button {
                            class: "mobile-menu-toggle",
                            r#type: "button",
                            aria_expanded: "{mobile_menu_expanded}",
                            aria_controls: "mobile-menu",
                            "aria-label": "{mobile_menu_label}",
                            onclick: toggle_mobile_menu,
                            svg {
                                class: "mobile-menu-icon",
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                if mobile_menu_is_open {
                                    line { x1: "18", y1: "6", x2: "6", y2: "18" }
                                    line { x1: "6", y1: "6", x2: "18", y2: "18" }
                                } else {
                                    line { x1: "3", y1: "6", x2: "21", y2: "6" }
                                    line { x1: "3", y1: "12", x2: "21", y2: "12" }
                                    line { x1: "3", y1: "18", x2: "21", y2: "18" }
                                }
                            }
                        }
                    }
                }
                div { class: "{mobile_backdrop_class}", aria_hidden: "true", onclick: close_mobile_menu }
                nav {
                    id: "mobile-menu",
                    class: "{mobile_menu_class}",
                    aria_label: "Mobile navigation",
                    aria_hidden: "{mobile_menu_hidden}",
                    a { class: "mobile-link", href: "/modules", onclick: close_mobile_menu, "Modules" }
                    {if is_authenticated {
                        rsx! {
                            a { class: "mobile-link", href: "/dashboard", onclick: close_mobile_menu, "Dashboard" }
                            div { class: "mobile-divider" }
                            div { class: "mobile-user",
                                span { class: "avatar-initial avatar-initial-sm", "{user_initial}" }
                                span { class: "mobile-user-name", "{user_handle}" }
                            }
                            form { action: "/auth/logout", method: "post",
                                button { class: "mobile-link mobile-button", r#type: "submit", "Log out" }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "mobile-divider" }
                            a {
                                class: "mobile-link mobile-button",
                                href: "{login_href}",
                                onclick: close_mobile_menu,
                                "Log In"
                            }
                        }
                    }}
                }
            }
            div {
                class: "{palette_backdrop_class}",
                role: "presentation",
                aria_hidden: "{palette_backdrop_hidden}",
                onclick: close_palette,
                div {
                    class: "palette",
                    role: "dialog",
                    aria_modal: "true",
                    aria_label: "Command palette",
                    tabindex: "-1",
                    onclick: move |event| event.stop_propagation(),
                    onkeydown: handle_palette_keydown,
                    div { class: "palette-header",
                        div { class: "search-wrapper",
                            svg {
                                class: "search-icon",
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                circle { cx: "11", cy: "11", r: "8" }
                                line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
                            }
                            input {
                                r#type: "text",
                                autocomplete: "off",
                                value: "{palette_query}",
                                oninput: {
                                    let palette = palette.clone();
                                    move |event| palette.set_query(event.value())
                                },
                                placeholder: "Find pages and commands...",
                                class: "search-input",
                                role: "combobox",
                                "aria-label": "Find pages and commands",
                                aria_haspopup: "listbox",
                                aria_expanded: if palette_has_results { "true" } else { "false" },
                                aria_controls: "palette-results",
                                aria_activedescendant: if palette_has_results {
                                    format!("palette-option-{palette_active_index}")
                                } else {
                                    String::new()
                                },
                                aria_autocomplete: "list",
                            }
                        }
                        div { class: "mode-chips",
                            for mode in [PaletteMode::All, PaletteMode::Pages, PaletteMode::Commands] {
                                button {
                                    class: if palette_mode == mode { "chip active" } else { "chip" },
                                    onclick: {
                                        let palette = palette.clone();
                                        move |_| palette.set_mode(mode)
                                    },
                                    "{palette_mode_label(mode)}"
                                }
                            }
                        }
                    }
                    div {
                        class: "palette-results",
                        role: "listbox",
                        id: "palette-results",
                        "aria-label": "Results",
                        if !palette_has_results {
                            div { class: "no-results", role: "status",
                                span { class: "no-results-text", "No results found" }
                            }
                        } else {
                            for (index, result) in palette_results.iter().enumerate() {
                                button {
                                    class: if index == palette_active_index {
                                        "result-item selected"
                                    } else {
                                        "result-item"
                                    },
                                    id: "palette-option-{index}",
                                    role: "option",
                                    aria_selected: if index == palette_active_index { "true" } else { "false" },
                                    "data-index": "{index}",
                                    onclick: {
                                        let execute_palette_item = execute_palette_item.clone();
                                        let item = result.item.clone();
                                        move |_| execute_palette_item(item.clone())
                                    },
                                    onmouseenter: {
                                        let mut palette_selected = palette_selected;
                                        move |_| palette_selected.set(index)
                                    },
                                    div { class: "result-icon",
                                        svg {
                                            width: "18",
                                            height: "18",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            path { d: "{result.item.icon.path()}" }
                                        }
                                    }
                                    div { class: "result-content",
                                        span { class: "result-name", "{result.item.name}" }
                                        if !result.item.description.is_empty() {
                                            span { class: "result-description", "{result.item.description}" }
                                        }
                                    }
                                    span { class: "result-type", "{palette_type_label(result.item.kind)}" }
                                }
                            }
                        }
                    }
                    div { class: "palette-footer",
                        div { class: "hint",
                            kbd { "↑" }
                            kbd { "↓" }
                            span { "Navigate" }
                        }
                        div { class: "hint",
                            kbd { "↵" }
                            span { "Select" }
                        }
                        div { class: "hint",
                            kbd { "Alt+Tab" }
                            span { "Filter" }
                        }
                        div { class: "hint",
                            kbd { "Esc" }
                            span { "Close" }
                        }
                    }
                }
            }
            main { id: "main-content", class: "app-main",
                {children}
            }
            footer { class: "site-footer",
                div { class: "footer-content",
                    a { class: "footer-brand", href: "/",
                        svg {
                            class: "footer-logo",
                            width: "20",
                            height: "20",
                            view_box: "0 0 100 100",
                            fill: "none",
                            defs {
                                linearGradient { id: "logoGradFooter", x1: "0%", y1: "0%", x2: "0%", y2: "100%",
                                    stop { offset: "0%", stop_color: "#cba6f7" }
                                    stop { offset: "100%", stop_color: "#89b4fa" }
                                }
                            }
                            path { d: "M10 20 L90 20 L78 38 L22 38 Z", fill: "url(#logoGradFooter)" }
                            path { d: "M30 43 L70 43 L70 60 L30 60 Z", fill: "url(#logoGradFooter)", fill_opacity: "0.9" }
                            path { d: "M22 65 L78 65 L90 85 L10 85 Z", fill: "url(#logoGradFooter)", fill_opacity: "0.8" }
                        }
                        span { class: "footer-name", "Barforge" }
                    }
                    nav { class: "footer-nav", aria_label: "Footer navigation",
                        a { href: "/terms", "Terms" }
                        a { href: "/privacy", "Privacy" }
                    }
                    div { class: "footer-license",
                        a { class: "github-link", href: "/barforge", aria_label: "View source on GitHub",
                            svg {
                                class: "footer-icon",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "currentColor",
                                path { d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" }
                            }
                        }
                        span { "GPL-3.0" }
                    }
                }
            }
        }
    }
}

#[component]
fn NotificationCenter() -> Element {
    let notifications = use_notifications();
    let is_open = use_signal(|| false);
    let unread = notifications.unread_count();
    let badge = if unread > 99 {
        "99+".to_string()
    } else {
        unread.to_string()
    };
    let dropdown_class = if *is_open.read() {
        "notification-dropdown open"
    } else {
        "notification-dropdown"
    };
    let aria_expanded = if *is_open.read() { "true" } else { "false" };
    let aria_hidden = if *is_open.read() { "false" } else { "true" };
    let visible = notifications
        .notifications()
        .into_iter()
        .filter(|notification| notification.status != NotificationStatus::Done)
        .collect::<Vec<_>>();
    let has_notifications = !visible.is_empty();

    let toggle = {
        let notifications = notifications.clone();
        move |_| {
            let next = !*is_open.read();
            let mut signal = is_open;
            signal.set(next);
            if next {
                notifications.sync_notifications();
            }
        }
    };
    let mark_all = {
        let notifications = notifications.clone();
        move |_| {
            notifications.mark_all_read_with_sync();
        }
    };

    rsx! {
        div { class: "notification-center",
            button {
                class: "header-icon notification-trigger",
                r#type: "button",
                aria_label: "Notifications",
                aria_expanded: "{aria_expanded}",
                onclick: toggle,
                svg {
                    class: "notification-bell-icon",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                    path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
                }
                {if unread > 0 {
                    rsx!(span { class: "notification-badge", "{badge}" })
                } else {
                    rsx! {}
                }}
            }
            div { class: "{dropdown_class}", aria_hidden: "{aria_hidden}",
                div { class: "notification-dropdown-header",
                    span { class: "notification-dropdown-title", "Notifications" }
                    {if unread > 0 {
                        rsx!(button {
                            class: "notification-mark-all",
                            r#type: "button",
                            onclick: mark_all,
                            "Mark all as read"
                        })
                    } else {
                        rsx! {}
                    }}
                }
                div { class: "notification-dropdown-body",
                    {if !has_notifications {
                        rsx!(
                            div { class: "notification-empty",
                                svg {
                                    class: "notification-bell-icon",
                                    width: "32",
                                    height: "32",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "aria-hidden": "true",
                                    path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                                    path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
                                }
                                p { "No notifications" }
                            }
                        )
                    } else {
                        rsx!(
                            ul { class: "notification-list",
                                for notification in visible {
                                    li {
                                        class: if notification.status == NotificationStatus::Unread {
                                            "notification-item unread"
                                        } else {
                                            "notification-item"
                                        },
                                        key: "{notification.id}",
                                        {if let Some(url) = notification.action_url.clone() {
                                            rsx!(a {
                                                class: "notification-link",
                                                href: "{url}",
                                                onclick: {
                                                    let notifications = notifications.clone();
                                                    let id = notification.id;
                                                    move |_| notifications.mark_read_with_sync(id)
                                                },
                                                span { class: "notification-title", "{notification.title}" }
                                                span { class: "notification-message", "{notification.body.clone().unwrap_or_default()}" }
                                            })
                                        } else {
                                            rsx!(button {
                                                class: "notification-link",
                                                r#type: "button",
                                                onclick: {
                                                    let notifications = notifications.clone();
                                                    let id = notification.id;
                                                    move |_| notifications.mark_read_with_sync(id)
                                                },
                                                span { class: "notification-title", "{notification.title}" }
                                                span { class: "notification-message", "{notification.body.clone().unwrap_or_default()}" }
                                            })
                                        }}
                                    }
                                }
                            }
                        )
                    }}
                }
            }
        }
    }
}

fn theme_label(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Light => "Light",
        ThemePreference::Dark => "Dark",
        ThemePreference::System => "System",
    }
}

fn palette_mode_label(mode: PaletteMode) -> &'static str {
    match mode {
        PaletteMode::All => "All",
        PaletteMode::Pages => "Pages",
        PaletteMode::Commands => "Commands",
    }
}

fn palette_type_label(kind: PaletteItemKind) -> &'static str {
    match kind {
        PaletteItemKind::Page => "Page",
        PaletteItemKind::Command => "Command",
    }
}

fn execute_palette_action(action: PaletteAction) {
    match action {
        PaletteAction::CopyUrl => copy_current_url(),
        PaletteAction::RefreshPage => refresh_page(),
    }
}

#[cfg(target_arch = "wasm32")]
fn open_external_url(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_external_url(_path: &str) {}

#[cfg(target_arch = "wasm32")]
fn set_body_scroll_locked(locked: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                let value = if locked { "hidden" } else { "" };
                let _ = body.style().set_property("overflow", value);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_body_scroll_locked(_locked: bool) {}

#[cfg(target_arch = "wasm32")]
fn copy_current_url() {
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            let clipboard = window.navigator().clipboard();
            let promise = clipboard.write_text(&href);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_current_url() {}

#[cfg(target_arch = "wasm32")]
fn refresh_page() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn refresh_page() {}

#[cfg(test)]
mod notification_center_tests {
    use super::*;
    use crate::notifications::{NotificationItem, NotificationStatus, use_notifications_provider};
    use barforge_types::NotificationType;

    #[test]
    fn notification_center_renders_empty_state() {
        let html = dioxus_ssr::render_element(rsx!(NotificationCenterFixture {
            notifications: Vec::new()
        }));

        assert!(html.contains("aria-label=\"Notifications\""));
        assert!(html.contains("No notifications"));
    }

    #[test]
    fn notification_center_shows_unread_badge() {
        let html = dioxus_ssr::render_element(rsx!(NotificationCenterFixture {
            notifications: vec![
                NotificationItem {
                    id: 1,
                    kind: NotificationType::Stars,
                    title: "New star".to_string(),
                    body: Some("Someone starred your module".to_string()),
                    action_url: Some("/modules/clock-time@barforge".to_string()),
                    created_at: "2025-01-01T00:00:00Z".to_string(),
                    status: NotificationStatus::Unread,
                },
                NotificationItem {
                    id: 2,
                    kind: NotificationType::Updates,
                    title: "Update shipped".to_string(),
                    body: Some("A new version is available".to_string()),
                    action_url: None,
                    created_at: "2025-01-01T00:10:00Z".to_string(),
                    status: NotificationStatus::Read,
                },
            ]
        }));

        assert!(html.contains("notification-badge"));
        assert!(html.contains(">1<"));
        assert!(html.contains("Mark all as read"));
    }

    #[component]
    fn NotificationCenterFixture(notifications: Vec<NotificationItem>) -> Element {
        let store = use_notifications_provider();
        store.set_notifications(notifications);

        rsx!(NotificationCenter {})
    }
}
