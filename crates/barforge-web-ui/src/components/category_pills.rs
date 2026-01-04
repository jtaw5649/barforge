use crate::categories::Category;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CategoryPillsProps {
    pub categories: Vec<Category>,
    pub base_url: Option<String>,
}

#[allow(non_snake_case)]
pub fn CategoryPills(
    CategoryPillsProps {
        categories,
        base_url,
    }: CategoryPillsProps,
) -> Element {
    let base_url = base_url.unwrap_or_else(|| "/modules/search".to_string());

    rsx! {
        div { class: "category-pills",
            {categories.into_iter().map(|category| {
                let Category { name, slug, icon, color } = category;
                let key = if slug.is_empty() { name } else { slug };
                let href = if slug.is_empty() {
                    base_url.clone()
                } else {
                    format!("{base_url}?category={slug}")
                };
                let icon = icon.map(|icon| {
                    rsx!(img {
                        class: "category-pill-icon",
                        src: icon,
                        alt: "",
                        width: "16",
                        height: "16"
                    })
                });
                rsx! {
                    a { key: "{key}", class: "category-pill", href: "{href}", style: "--cat-color: {color}",
                        {icon}
                        span { class: "category-pill-name", "{name}" }
                    }
                }
            })}
        }
    }
}
