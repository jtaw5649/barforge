use barforge_web::command_palette::{PaletteItemKind, palette_items};

#[test]
fn palette_filters_requires_auth_and_admin() {
    let anonymous = palette_items(false, false);
    assert!(anonymous.iter().any(|item| item.id == "login"));
    assert!(!anonymous.iter().any(|item| item.id == "dashboard"));
    assert!(!anonymous.iter().any(|item| item.id == "admin"));

    let user = palette_items(true, false);
    assert!(!user.iter().any(|item| item.id == "login"));
    assert!(user.iter().any(|item| item.id == "dashboard"));
    assert!(!user.iter().any(|item| item.id == "admin"));

    let admin = palette_items(true, true);
    assert!(admin.iter().any(|item| item.id == "admin"));
}

#[test]
fn palette_includes_commands_and_pages() {
    let items = palette_items(true, true);
    assert!(items.iter().any(|item| item.kind == PaletteItemKind::Page));
    assert!(
        items
            .iter()
            .any(|item| item.kind == PaletteItemKind::Command)
    );
}
