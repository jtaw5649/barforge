use barforge_web::theme::{ThemeMode, ThemePreference};

#[test]
fn theme_cycles_system_light_dark() {
    let theme = ThemePreference::System;
    let theme = theme.cycle();
    assert_eq!(theme, ThemePreference::Light);
    let theme = theme.cycle();
    assert_eq!(theme, ThemePreference::Dark);
    let theme = theme.cycle();
    assert_eq!(theme, ThemePreference::System);
}

#[test]
fn theme_effective_respects_system_mode() {
    assert_eq!(
        ThemePreference::System.effective(ThemeMode::Dark),
        ThemeMode::Dark
    );
    assert_eq!(
        ThemePreference::System.effective(ThemeMode::Light),
        ThemeMode::Light
    );
}

#[test]
fn theme_parse_defaults_to_system_for_unknown() {
    assert_eq!(
        ThemePreference::from_storage(Some("unknown")),
        ThemePreference::System
    );
}
