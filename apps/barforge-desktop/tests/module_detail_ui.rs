use barforge_desktop::state::InstallStage;
use barforge_desktop::ui::ModuleDetailShell;
use barforge_types::{Review, ReviewUser, ReviewsResponse};
use dioxus::prelude::*;

fn make_reviews() -> ReviewsResponse {
    ReviewsResponse {
        reviews: vec![Review {
            id: 1,
            rating: 5,
            title: Some("Great module".to_string()),
            body: Some("Fast and reliable".to_string()),
            helpful_count: 2,
            user: ReviewUser {
                username: "alice".to_string(),
                avatar_url: None,
            },
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: None,
        }],
        total: 1,
    }
}

#[test]
fn module_detail_renders_reviews() {
    let reviews = make_reviews();

    let html = dioxus_ssr::render_element(rsx!(ModuleDetailShell {
        reviews: reviews.clone(),
        installing: false,
        install_stage: None,
    }));

    assert!(html.contains("Great module"));
    assert!(html.contains("Fast and reliable"));
    assert!(html.contains("alice"));
    assert!(html.contains("5"));
}

#[test]
fn module_detail_renders_install_stage() {
    let reviews = ReviewsResponse {
        reviews: Vec::new(),
        total: 0,
    };

    let html = dioxus_ssr::render_element(rsx!(ModuleDetailShell {
        reviews,
        installing: true,
        install_stage: Some(InstallStage::DownloadingPackage),
    }));

    assert!(html.contains("Downloading package"));
}
