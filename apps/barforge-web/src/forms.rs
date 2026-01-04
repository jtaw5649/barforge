use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldError {
    pub(crate) field: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct UploadFormValues {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) repo_url: String,
    pub(crate) version: String,
    pub(crate) license: String,
    pub(crate) package_file: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProfileFormValues {
    pub(crate) display_name: String,
    pub(crate) bio: String,
    pub(crate) website_url: String,
    pub(crate) github_url: String,
    pub(crate) twitter_url: String,
    pub(crate) bluesky_url: String,
    pub(crate) discord_url: String,
    pub(crate) sponsor_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct UpdateProfileRequest {
    pub(crate) display_name: Option<String>,
    pub(crate) bio: Option<String>,
    pub(crate) website_url: Option<String>,
    pub(crate) github_url: Option<String>,
    pub(crate) twitter_url: Option<String>,
    pub(crate) bluesky_url: Option<String>,
    pub(crate) discord_url: Option<String>,
    pub(crate) sponsor_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UploadRequestSpec {
    pub(crate) upload_endpoint: &'static str,
    pub(crate) csrf_endpoint: &'static str,
    pub(crate) csrf_header: &'static str,
    pub(crate) turnstile_field: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProfileRequestSpec {
    pub(crate) profile_endpoint: &'static str,
    pub(crate) csrf_endpoint: &'static str,
    pub(crate) csrf_header: &'static str,
}

pub(crate) fn upload_request_spec() -> UploadRequestSpec {
    UploadRequestSpec {
        upload_endpoint: "/api/upload",
        csrf_endpoint: "/api/csrf-token",
        csrf_header: "x-csrf-token",
        turnstile_field: "cf-turnstile-response",
    }
}

pub(crate) fn profile_request_spec() -> ProfileRequestSpec {
    ProfileRequestSpec {
        profile_endpoint: "/api/users/me",
        csrf_endpoint: "/api/csrf-token",
        csrf_header: "x-csrf-token",
    }
}

pub(crate) fn validate_upload_form(values: &UploadFormValues) -> Vec<FieldError> {
    let mut errors = Vec::new();

    if values.name.trim().is_empty() {
        errors.push(FieldError {
            field: "name",
            message: "Module name is required.",
        });
    }
    if values.description.trim().is_empty() {
        errors.push(FieldError {
            field: "description",
            message: "Description is required.",
        });
    }
    if values.category.trim().is_empty() {
        errors.push(FieldError {
            field: "category",
            message: "Category is required.",
        });
    }
    if values.repo_url.trim().is_empty() {
        errors.push(FieldError {
            field: "repo_url",
            message: "Repository URL is required.",
        });
    } else if !values.repo_url.starts_with("https://") {
        errors.push(FieldError {
            field: "repo_url",
            message: "Repository URL must start with https://",
        });
    }
    if values.version.trim().is_empty() {
        errors.push(FieldError {
            field: "version",
            message: "Version is required.",
        });
    }
    if values.license.trim().is_empty() {
        errors.push(FieldError {
            field: "license",
            message: "License is required.",
        });
    }
    if values.package_file.trim().is_empty() {
        errors.push(FieldError {
            field: "package_file",
            message: "Package file is required.",
        });
    }

    errors
}

pub(crate) fn validate_profile_form(values: &ProfileFormValues) -> Vec<FieldError> {
    let mut errors = Vec::new();

    if values.display_name.trim().chars().count() > 50 {
        errors.push(FieldError {
            field: "display_name",
            message: "Display name must be 50 characters or less",
        });
    }
    if values.bio.trim().chars().count() > 500 {
        errors.push(FieldError {
            field: "bio",
            message: "Bio must be 500 characters or less",
        });
    }

    for (field, value) in [
        ("website_url", values.website_url.as_str()),
        ("github_url", values.github_url.as_str()),
        ("twitter_url", values.twitter_url.as_str()),
        ("bluesky_url", values.bluesky_url.as_str()),
        ("discord_url", values.discord_url.as_str()),
        ("sponsor_url", values.sponsor_url.as_str()),
    ] {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if Url::parse(trimmed).is_err() {
            errors.push(FieldError {
                field,
                message: "Must be a valid URL",
            });
        }
    }

    errors
}

pub(crate) fn upload_error_summary(errors: &[FieldError]) -> Element {
    rsx! {
        div {
            class: "form-error-summary",
            role: "alert",
            "aria-live": "assertive",
            h3 { "Please correct the highlighted fields" }
            ul { class: "form-error-list",
                {errors.iter().map(|error| rsx!(
                    li { key: "{error.field}", "{error.message}" }
                ))}
            }
        }
    }
}

pub(crate) fn field_error_message(errors: &[FieldError], field: &str) -> Option<&'static str> {
    errors
        .iter()
        .find(|error| error.field == field)
        .map(|error| error.message)
}

pub(crate) fn field_error_element(id: &str, message: Option<&'static str>) -> Element {
    let (visible, hidden, text) = match message {
        Some(message) => ("true", "false", message),
        None => ("false", "true", ""),
    };

    rsx! {
        p {
            id: "{id}",
            class: "form-error",
            "data-visible": "{visible}",
            "aria-hidden": "{hidden}",
            "{text}"
        }
    }
}

pub(crate) fn upload_form_values(event: &FormEvent) -> UploadFormValues {
    let values = event.values();
    UploadFormValues {
        name: form_value(&values, "name"),
        description: form_value(&values, "description"),
        category: form_value(&values, "category"),
        repo_url: form_value(&values, "repo_url"),
        version: form_value(&values, "version"),
        license: form_value(&values, "license"),
        package_file: form_value(&values, "package_file"),
    }
}

pub(crate) fn profile_form_values(event: &FormEvent) -> ProfileFormValues {
    let values = event.values();
    ProfileFormValues {
        display_name: form_value(&values, "display_name"),
        bio: form_value(&values, "bio"),
        website_url: form_value(&values, "website_url"),
        github_url: form_value(&values, "github_url"),
        twitter_url: form_value(&values, "twitter_url"),
        bluesky_url: form_value(&values, "bluesky_url"),
        discord_url: form_value(&values, "discord_url"),
        sponsor_url: form_value(&values, "sponsor_url"),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn update_profile_request(values: &ProfileFormValues) -> UpdateProfileRequest {
    fn to_option(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    UpdateProfileRequest {
        display_name: to_option(&values.display_name),
        bio: to_option(&values.bio),
        website_url: to_option(&values.website_url),
        github_url: to_option(&values.github_url),
        twitter_url: to_option(&values.twitter_url),
        bluesky_url: to_option(&values.bluesky_url),
        discord_url: to_option(&values.discord_url),
        sponsor_url: to_option(&values.sponsor_url),
    }
}

pub(crate) fn form_value(values: &[(String, FormValue)], key: &str) -> String {
    values
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| match value {
            FormValue::Text(value) => value.clone(),
            FormValue::File(file) => file.as_ref().map(|file| file.name()).unwrap_or_default(),
        })
        .unwrap_or_default()
}
