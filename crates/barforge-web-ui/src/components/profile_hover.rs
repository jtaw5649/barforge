use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ProfileHoverProps {
    pub username: String,
    #[props(optional)]
    pub label: Option<String>,
    #[props(optional)]
    pub avatar_url: Option<String>,
    #[props(optional)]
    pub link_class: Option<String>,
}

#[allow(non_snake_case)]
pub fn ProfileHover(
    ProfileHoverProps {
        username,
        label,
        avatar_url,
        link_class,
    }: ProfileHoverProps,
) -> Element {
    let label = label.unwrap_or_else(|| username.clone());
    let link_class = link_class.unwrap_or_default();
    let avatar = avatar_url.as_ref().map(|url| {
        rsx!(img {
            class: "profile-hover-avatar",
            src: "{url}",
            alt: "",
            width: "28",
            height: "28",
            loading: "lazy",
        })
    });

    rsx! {
        div { class: "profile-hover",
            a { class: "{link_class}", href: "/users/{username}", "{label}" }
            div { class: "profile-hover-card",
                div { class: "profile-hover-header",
                    {avatar}
                    span { class: "profile-hover-name", "{username}" }
                }
                span { class: "profile-hover-action", "View profile ->" }
            }
        }
    }
}
