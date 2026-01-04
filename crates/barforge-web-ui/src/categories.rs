use barforge_types::ModuleCategory;
use manganis::{Asset, asset};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Category {
    pub name: &'static str,
    pub slug: &'static str,
    pub icon: Option<Asset>,
    pub color: &'static str,
}

const CATEGORY_ALL_ICON: Asset = asset!("/assets/icons/category-all.svg");
const CATEGORY_SYSTEM_ICON: Asset = asset!("/assets/icons/category-system.svg");
const CATEGORY_HARDWARE_ICON: Asset = asset!("/assets/icons/category-hardware.svg");
const CATEGORY_NETWORK_ICON: Asset = asset!("/assets/icons/category-network.svg");
const CATEGORY_AUDIO_ICON: Asset = asset!("/assets/icons/category-audio.svg");
const CATEGORY_POWER_ICON: Asset = asset!("/assets/icons/category-power.svg");
const CATEGORY_TIME_ICON: Asset = asset!("/assets/icons/category-time.svg");
const CATEGORY_CLOCK_ICON: Asset = asset!("/assets/icons/category-clock.svg");
const CATEGORY_WORKSPACE_ICON: Asset = asset!("/assets/icons/category-workspace.svg");
const CATEGORY_WINDOW_ICON: Asset = asset!("/assets/icons/category-window.svg");
const CATEGORY_TRAY_ICON: Asset = asset!("/assets/icons/category-tray.svg");
const CATEGORY_WEATHER_ICON: Asset = asset!("/assets/icons/category-weather.svg");
const CATEGORY_PRODUCTIVITY_ICON: Asset = asset!("/assets/icons/category-productivity.svg");
const CATEGORY_MEDIA_ICON: Asset = asset!("/assets/icons/category-media.svg");
const CATEGORY_CUSTOM_ICON: Asset = asset!("/assets/icons/category-custom.svg");

const CATEGORIES: [Category; 14] = [
    Category {
        name: "System",
        slug: "system",
        icon: Some(CATEGORY_SYSTEM_ICON),
        color: "#617dfa",
    },
    Category {
        name: "Hardware",
        slug: "hardware",
        icon: Some(CATEGORY_HARDWARE_ICON),
        color: "#10b981",
    },
    Category {
        name: "Network",
        slug: "network",
        icon: Some(CATEGORY_NETWORK_ICON),
        color: "#f59e0b",
    },
    Category {
        name: "Audio",
        slug: "audio",
        icon: Some(CATEGORY_AUDIO_ICON),
        color: "#f97316",
    },
    Category {
        name: "Power",
        slug: "power",
        icon: Some(CATEGORY_POWER_ICON),
        color: "#22c55e",
    },
    Category {
        name: "Time",
        slug: "time",
        icon: Some(CATEGORY_TIME_ICON),
        color: "#3b82f6",
    },
    Category {
        name: "Clock",
        slug: "clock",
        icon: Some(CATEGORY_CLOCK_ICON),
        color: "#06b6d4",
    },
    Category {
        name: "Workspace",
        slug: "workspace",
        icon: Some(CATEGORY_WORKSPACE_ICON),
        color: "#8b5cf6",
    },
    Category {
        name: "Window",
        slug: "window",
        icon: Some(CATEGORY_WINDOW_ICON),
        color: "#a855f7",
    },
    Category {
        name: "Tray",
        slug: "tray",
        icon: Some(CATEGORY_TRAY_ICON),
        color: "#6366f1",
    },
    Category {
        name: "Weather",
        slug: "weather",
        icon: Some(CATEGORY_WEATHER_ICON),
        color: "#0ea5e9",
    },
    Category {
        name: "Productivity",
        slug: "productivity",
        icon: Some(CATEGORY_PRODUCTIVITY_ICON),
        color: "#14b8a6",
    },
    Category {
        name: "Media",
        slug: "media",
        icon: Some(CATEGORY_MEDIA_ICON),
        color: "#ec4899",
    },
    Category {
        name: "Custom",
        slug: "custom",
        icon: Some(CATEGORY_CUSTOM_ICON),
        color: "#64748b",
    },
];

pub fn homepage_categories() -> Vec<Category> {
    let featured = [
        "system",
        "hardware",
        "network",
        "media",
        "workspace",
        "clock",
    ];
    CATEGORIES
        .iter()
        .copied()
        .filter(|category| featured.contains(&category.slug))
        .collect()
}

pub fn browse_categories() -> Vec<Category> {
    let mut categories = Vec::with_capacity(CATEGORIES.len() + 1);
    categories.push(Category {
        name: "All",
        slug: "",
        icon: Some(CATEGORY_ALL_ICON),
        color: "#64748b",
    });
    categories.extend(CATEGORIES.iter().copied());
    categories
}

pub fn category_label(category: &ModuleCategory) -> &'static str {
    match category {
        ModuleCategory::System => "System",
        ModuleCategory::Hardware => "Hardware",
        ModuleCategory::Network => "Network",
        ModuleCategory::Audio => "Audio",
        ModuleCategory::Power => "Power",
        ModuleCategory::Time => "Time",
        ModuleCategory::Workspace => "Workspace",
        ModuleCategory::Window => "Window",
        ModuleCategory::Tray => "Tray",
        ModuleCategory::Weather => "Weather",
        ModuleCategory::Productivity => "Productivity",
        ModuleCategory::Media => "Media",
        ModuleCategory::Custom => "Custom",
    }
}

pub fn category_color(category: &ModuleCategory) -> &'static str {
    let label = category_label(category);
    CATEGORIES
        .iter()
        .find(|entry| entry.name == label)
        .map(|entry| entry.color)
        .unwrap_or("#64748b")
}

#[cfg(test)]
mod tests {
    use super::*;
    use barforge_types::ModuleCategory;

    #[test]
    fn category_label_matches_module_category() {
        assert_eq!(category_label(&ModuleCategory::System), "System");
        assert_eq!(category_label(&ModuleCategory::Weather), "Weather");
        assert_eq!(category_label(&ModuleCategory::Custom), "Custom");
    }

    #[test]
    fn category_color_matches_module_category() {
        assert_eq!(category_color(&ModuleCategory::System), "#617dfa");
        assert_eq!(category_color(&ModuleCategory::Weather), "#0ea5e9");
        assert_eq!(category_color(&ModuleCategory::Custom), "#64748b");
    }
}
