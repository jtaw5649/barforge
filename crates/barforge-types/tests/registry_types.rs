use std::{collections::BTreeMap, path::PathBuf};

use jsonschema::Resource;
use serde_json::{Value, json};

use barforge_types::{
    ApiVersion, CategoriesResponse, CategoryInfo, ModuleCategory, ModulesResponse, RegistryIndex,
    RegistryModule, VersionHistoryEntry, VersionsResponse,
};

#[test]
fn registry_module_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "RegistryModule");

    let instance = serde_json::to_value(sample_registry_module())
        .expect("failed to serialize registry module");

    assert_schema_match(&validator, &instance, "RegistryModule");
}

#[test]
fn modules_response_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "ModulesResponse");

    let instance = serde_json::to_value(ModulesResponse {
        modules: vec![sample_registry_module()],
        total: 1,
    })
    .expect("failed to serialize modules response");

    assert_schema_match(&validator, &instance, "ModulesResponse");
}

#[test]
fn category_info_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "CategoryInfo");

    let instance = serde_json::to_value(CategoryInfo {
        id: Some("system".to_string()),
        name: "System".to_string(),
        icon: "cpu".to_string(),
    })
    .expect("failed to serialize category info");

    assert_schema_match(&validator, &instance, "CategoryInfo");
}

#[test]
fn categories_response_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "CategoriesResponse");

    let instance = serde_json::to_value(CategoriesResponse {
        categories: vec![CategoryInfo {
            id: None,
            name: "Weather".to_string(),
            icon: "cloud".to_string(),
        }],
    })
    .expect("failed to serialize categories response");

    assert_schema_match(&validator, &instance, "CategoriesResponse");
}

#[test]
fn api_version_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "ApiVersion");

    let instance =
        serde_json::to_value(ApiVersion { version: 1 }).expect("failed to serialize api version");

    assert_schema_match(&validator, &instance, "ApiVersion");
}

#[test]
fn registry_index_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "RegistryIndex");

    let categories = BTreeMap::from([(
        "system".to_string(),
        CategoryInfo {
            id: Some("system".to_string()),
            name: "System".to_string(),
            icon: "cpu".to_string(),
        },
    )]);

    let instance = serde_json::to_value(RegistryIndex {
        version: 1,
        modules: vec![sample_registry_module()],
        categories,
    })
    .expect("failed to serialize registry index");

    assert_schema_match(&validator, &instance, "RegistryIndex");
}

#[test]
fn version_history_entry_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "VersionHistoryEntry");

    let instance = serde_json::to_value(sample_version_history_entry())
        .expect("failed to serialize version history entry");

    assert_schema_match(&validator, &instance, "VersionHistoryEntry");
}

#[test]
fn versions_response_serializes_to_openapi_schema() {
    let spec_json = load_openapi_json(&openapi_path());
    let validator = build_validator(&spec_json, "VersionsResponse");

    let instance = serde_json::to_value(VersionsResponse {
        versions: vec![sample_version_history_entry()],
        total: 1,
    })
    .expect("failed to serialize versions response");

    assert_schema_match(&validator, &instance, "VersionsResponse");
}

fn sample_registry_module() -> RegistryModule {
    RegistryModule {
        uuid: "weather-wttr@barforge".to_string(),
        name: "Weather".to_string(),
        description: "Weather module".to_string(),
        author: "barforge".to_string(),
        category: ModuleCategory::Weather,
        icon: Some("https://example.com/icon.png".to_string()),
        screenshot: Some("https://example.com/screenshot.png".to_string()),
        repo_url: "https://github.com/barforge/weather".to_string(),
        downloads: 12,
        version: Some("1.0.0".to_string()),
        last_updated: Some("2024-01-02T03:04:05Z".to_string()),
        rating: Some(4.5_f32),
        verified_author: true,
        tags: vec!["weather".to_string()],
        checksum: Some("sha256:abcd".to_string()),
        license: Some("MIT".to_string()),
    }
}

fn sample_version_history_entry() -> VersionHistoryEntry {
    VersionHistoryEntry {
        version: "1.2.3".to_string(),
        changelog: Some("Added new widgets.".to_string()),
        downloads: 123,
        published_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

fn build_validator(spec_json: &Value, schema_name: &str) -> jsonschema::Validator {
    let schema = json!({
        "$ref": format!("@@root#/components/schemas/{schema_name}"),
    });

    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_resource("@@root", Resource::from_contents(spec_json.clone()))
        .build(&schema)
        .expect("failed to build schema validator")
}

fn assert_schema_match(validator: &jsonschema::Validator, instance: &Value, schema_name: &str) {
    if let Err(error) = validator.validate(instance) {
        panic!("{schema_name} did not match schema: {error}");
    }
}

fn openapi_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../../barforge-registry-api/docs/openapi.yaml")
}

fn load_openapi_json(path: &PathBuf) -> Value {
    let content = std::fs::read_to_string(path).expect("failed to read openapi spec");
    let yaml_value: serde_yml::Value =
        serde_yml::from_str(&content).expect("failed to parse openapi yaml");
    serde_json::to_value(yaml_value).expect("failed to convert spec to json")
}
