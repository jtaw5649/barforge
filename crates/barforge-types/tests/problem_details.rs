use std::path::PathBuf;

use jsonschema::Resource;
use serde_json::{Value, json};

use barforge_types::ProblemDetails;

#[test]
fn problem_details_serializes_to_openapi_schema() {
    let openapi_path = openapi_path();
    let spec_json = load_openapi_json(&openapi_path);

    let schema = json!({
        "$ref": "@@root#/components/schemas/ProblemDetails"
    });

    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_resource("@@root", Resource::from_contents(spec_json))
        .build(&schema)
        .expect("failed to build schema validator");

    let instance = serde_json::to_value(ProblemDetails {
        type_: "about:blank".to_string(),
        title: "Not Found".to_string(),
        status: 404,
        detail: "Module not found".to_string(),
        instance: None,
        code: Some("MODULE_NOT_FOUND".to_string()),
        error_id: Some("test-correlation-id".to_string()),
    })
    .expect("failed to serialize problem details");

    if let Err(error) = validator.validate(&instance) {
        panic!("problem details did not match schema: {error}");
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
