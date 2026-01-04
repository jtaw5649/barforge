use barforge_desktop::state::{
    InstallStage, ModuleDetailState, ReviewsLoadingState, ScreenshotState,
};
use barforge_types::ReviewsResponse;

#[test]
fn screenshot_load_sets_loading_when_available() {
    let mut state = ModuleDetailState::default();

    state.begin_screenshot_load(true);

    assert_eq!(state.screenshot, ScreenshotState::Loading);
}

#[test]
fn screenshot_load_sets_not_loaded_when_missing() {
    let mut state = ModuleDetailState::default();

    state.begin_screenshot_load(false);

    assert_eq!(state.screenshot, ScreenshotState::NotLoaded);
}

#[test]
fn screenshot_load_success_sets_loaded() {
    let mut state = ModuleDetailState::default();

    state.begin_screenshot_load(true);
    state.handle_screenshot_loaded(Ok("screenshot".to_string()));

    assert_eq!(
        state.screenshot,
        ScreenshotState::Loaded("screenshot".to_string())
    );
}

#[test]
fn screenshot_load_failure_sets_failed() {
    let mut state = ModuleDetailState::default();

    state.begin_screenshot_load(true);
    state.handle_screenshot_loaded(Err(()));

    assert_eq!(state.screenshot, ScreenshotState::Failed);
}

#[test]
fn screenshot_reset_returns_to_not_loaded() {
    let mut state = ModuleDetailState::default();

    state.begin_screenshot_load(true);
    state.handle_screenshot_loaded(Ok("screenshot".to_string()));
    state.reset();

    assert_eq!(state.screenshot, ScreenshotState::NotLoaded);
}

#[test]
fn reviews_load_success_sets_loaded() {
    let mut state = ModuleDetailState::default();
    let reviews = ReviewsResponse {
        reviews: Vec::new(),
        total: 0,
    };

    state.begin_reviews_load();
    state.handle_reviews_loaded(Ok(reviews.clone()));

    assert_eq!(state.reviews, ReviewsLoadingState::Loaded(reviews));
}

#[test]
fn reviews_load_failure_sets_failed() {
    let mut state = ModuleDetailState::default();

    state.begin_reviews_load();
    state.handle_reviews_loaded(Err("fetch failed".to_string()));

    assert_eq!(
        state.reviews,
        ReviewsLoadingState::Failed("fetch failed".to_string())
    );
}

#[test]
fn install_stage_transitions_success_preserve_stage() {
    let mut state = ModuleDetailState::default();

    state.start_install();
    state.set_install_stage(InstallStage::DownloadingPackage);
    state.finish_install(true);

    assert!(!state.installing);
    assert_eq!(state.install_stage, Some(InstallStage::DownloadingPackage));
}

#[test]
fn install_stage_failure_clears_stage() {
    let mut state = ModuleDetailState::default();

    state.start_install();
    state.set_install_stage(InstallStage::VerifyingHash);
    state.finish_install(false);

    assert!(!state.installing);
    assert_eq!(state.install_stage, None);
}
