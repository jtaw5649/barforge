use std::path::PathBuf;

#[test]
fn openapi_contains_first_slice_endpoints() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let openapi_path = manifest_dir.join("../../../barforge-registry-api/docs/openapi.yaml");

    let spec = barforge_contracts::openapi::load_openapi(&openapi_path)
        .expect("failed to load openapi spec");

    let paths = spec
        .get("paths")
        .and_then(|value| value.as_mapping())
        .expect("openapi spec missing paths mapping");

    for endpoint in [
        "/api/v1/index",
        "/api/v1/featured",
        "/api/v1/landing",
        "/api/v1/modules",
        "/api/v1/modules/{uuid}",
        "/api/v1/modules/{uuid}/related",
        "/api/v1/categories",
    ] {
        let key = serde_yml::Value::String(endpoint.to_string());
        assert!(
            paths.contains_key(&key),
            "missing required endpoint: {endpoint}"
        );
    }
}

#[test]
fn openapi_first_slice_responses_define_json_schema() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let openapi_path = manifest_dir.join("../../../barforge-registry-api/docs/openapi.yaml");

    let spec = barforge_contracts::openapi::load_openapi_spec(&openapi_path)
        .expect("failed to load openapi spec");

    let paths = spec.paths.as_ref().expect("openapi spec missing paths");

    for endpoint in [
        "/api/v1/index",
        "/api/v1/featured",
        "/api/v1/landing",
        "/api/v1/modules",
        "/api/v1/modules/{uuid}",
        "/api/v1/modules/{uuid}/related",
        "/api/v1/categories",
    ] {
        let path_item = paths.get(endpoint).expect("missing required endpoint");
        let operation = path_item.get.as_ref().expect("missing GET operation");
        let responses = operation.responses(&spec);
        let response = responses.get("200").expect("missing 200 response");
        let media_type = response
            .content
            .get("application/json")
            .expect("missing application/json response");

        assert!(
            media_type
                .schema(&spec)
                .expect("failed to resolve response schema")
                .is_some(),
            "missing response schema for {endpoint}"
        );
    }
}
