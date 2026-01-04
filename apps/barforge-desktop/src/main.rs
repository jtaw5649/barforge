#[cfg(feature = "desktop")]
fn main() {
    dioxus::launch(barforge_desktop::ui::DesktopEntry);
}

#[cfg(not(feature = "desktop"))]
fn main() {}
