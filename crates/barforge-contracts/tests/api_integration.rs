use std::path::PathBuf;

use http::Request;
use jsonschema::Resource;
use oasert::validator::OpenApiPayloadValidator;
use serde_json::{Value, json};
use ureq::RequestExt;

#[test]
fn modules_list_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules?limit=1");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /api/v1/modules failed");
    assert_eq!(status, 200, "unexpected status for modules list");

    validate_response_schema(&spec_json, "/api/v1/modules", &response_json);
}

#[test]
fn modules_list_invalid_limit_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules?limit=not-a-number");
    let (status, content_type, response_json) = send_empty_request_json_response("GET", &url)
        .expect("GET /api/v1/modules invalid limit failed");
    assert_eq!(status, 400, "expected 400 for invalid limit");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn modules_list_invalid_offset_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules?limit=1&offset=not-a-number");
    let (status, content_type, response_json) = send_empty_request_json_response("GET", &url)
        .expect("GET /api/v1/modules invalid offset failed");
    assert_eq!(status, 400, "expected 400 for invalid offset");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn module_create_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping module create test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping module create test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(payload) = module_create_payload() else {
        eprintln!("Skipping module create test; set BARFORGE_API_MODULE_CREATE_JSON to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, response_json) = send_json_response("POST", &url, &payload, Some(&token))
        .expect("POST /api/v1/modules failed");
    assert_eq!(status, 200, "unexpected status for module create");

    validate_response_schema_for_method(&spec_json, "/api/v1/modules", "post", &response_json);
}

#[test]
fn module_create_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping module create invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping module create invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }

    let payload = json!({});
    let url = format!("{base_url}/api/v1/modules");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/modules invalid payload failed");
    assert_eq!(
        status, 400,
        "expected 400 for invalid module create payload"
    );
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn collections_create_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collections create test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections create test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(payload) = collection_create_payload() else {
        eprintln!(
            "Skipping collections create test; set BARFORGE_API_COLLECTION_CREATE_JSON to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/collections");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/collections failed");
    assert_eq!(status, 200, "unexpected status for collections create");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/collections",
        "post",
        content_type.as_deref(),
    );
    validate_response_schema_for_method(&spec_json, "/api/v1/collections", "post", &response_json);
}

#[test]
fn collections_create_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping collections create invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections create invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }

    let payload = json!({});
    let url = format!("{base_url}/api/v1/collections");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/collections invalid payload failed");
    assert_eq!(status, 400, "expected 400 for invalid collection payload");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn collections_update_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collections update test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections update test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(payload) = collection_update_payload() else {
        eprintln!(
            "Skipping collections update test; set BARFORGE_API_COLLECTION_UPDATE_JSON to enable."
        );
        return;
    };
    let Some(id) = collection_id(&base_url, &token) else {
        eprintln!("Skipping collections update test; set BARFORGE_API_COLLECTION_ID to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/collections/{id}");
    validate_json_request(&request_validator, "PATCH", &url, Some(&token), &payload);
    let (status, content_type, body_len) =
        send_json_text_response("PATCH", &url, &payload, Some(&token))
            .expect("PATCH /api/v1/collections/{id} failed");
    assert_eq!(status, 200, "unexpected status for collections update");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/collections/{id}",
        "patch",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected collections update response body");
}

#[test]
fn collections_update_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping collections update invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections update invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(id) = collection_id(&base_url, &token) else {
        eprintln!(
            "Skipping collections update invalid payload test; set BARFORGE_API_COLLECTION_ID to enable."
        );
        return;
    };

    let payload = json!({ "visibility": "not-valid" });
    let url = format!("{base_url}/api/v1/collections/{id}");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("PATCH", &url, &payload, Some(&token))
            .expect("PATCH /api/v1/collections/{id} invalid payload failed");
    assert_eq!(
        status, 400,
        "expected 400 for invalid collection update payload"
    );
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn collections_add_module_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collections add module test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections add module test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(id) = collection_add_id(&base_url, &token) else {
        eprintln!(
            "Skipping collections add module test; set BARFORGE_API_COLLECTION_ADD_ID to enable."
        );
        return;
    };
    let Some(payload) = collection_add_module_payload() else {
        eprintln!(
            "Skipping collections add module test; set BARFORGE_API_COLLECTION_ADD_MODULE_JSON to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/collections/{id}/modules");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, content_type, body_len) =
        send_json_text_response("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/collections/{id}/modules failed");
    assert_eq!(status, 200, "unexpected status for collections add module");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/collections/{id}/modules",
        "post",
        content_type.as_deref(),
    );
    assert!(
        body_len > 0,
        "expected collections add module response body"
    );
}

#[test]
fn collections_add_module_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping collections add module invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping collections add module invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(id) = collection_id(&base_url, &token) else {
        eprintln!(
            "Skipping collections add module invalid payload test; set BARFORGE_API_COLLECTION_ID to enable."
        );
        return;
    };

    let payload = json!({});
    let url = format!("{base_url}/api/v1/collections/{id}/modules");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/collections/{id}/modules invalid payload failed");
    assert_eq!(status, 400, "expected 400 for invalid add module payload");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn collections_remove_module_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping collections remove module test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_destructive() {
        eprintln!(
            "Skipping collections remove module test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable."
        );
        return;
    }
    let Some((id, uuid)) = collection_remove_params() else {
        eprintln!(
            "Skipping collections remove module test; set BARFORGE_API_COLLECTION_REMOVE_ID and BARFORGE_API_COLLECTION_REMOVE_UUID to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/collections/{id}/modules/{uuid}");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));
    let (status, content_type, body_len) =
        send_empty_request_text_response("DELETE", &url, Some(&token))
            .expect("DELETE /api/v1/collections/{id}/modules/{uuid} failed");
    assert_eq!(
        status, 200,
        "unexpected status for collections remove module"
    );
    assert_expected_content_type(
        &spec_json,
        "/api/v1/collections/{id}/modules/{uuid}",
        "delete",
        content_type.as_deref(),
    );
    assert!(
        body_len > 0,
        "expected collections remove module response body"
    );
}

#[test]
fn collections_delete_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collections delete test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!(
            "Skipping collections delete test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable."
        );
        return;
    }
    let Some(id) = collection_delete_id() else {
        eprintln!(
            "Skipping collections delete test; set BARFORGE_API_COLLECTION_DELETE_ID to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/collections/{id}");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));
    let (status, content_type, body_len) =
        send_empty_request_text_response("DELETE", &url, Some(&token))
            .expect("DELETE /api/v1/collections/{id} failed");
    assert_eq!(status, 200, "unexpected status for collections delete");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/collections/{id}",
        "delete",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected collections delete response body");
}

#[test]
fn modules_search_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules/search?q=clock&limit=1");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/search failed");
    assert_eq!(status, 200, "unexpected status for modules search");

    validate_response_schema(&spec_json, "/api/v1/modules/search", &response_json);
}

#[test]
fn modules_search_invalid_limit_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules/search?q=clock&limit=not-a-number");
    let (status, content_type, response_json) = send_empty_request_json_response("GET", &url)
        .expect("GET /api/v1/modules/search invalid limit failed");
    assert_eq!(status, 400, "expected 400 for invalid search limit");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn index_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/index");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /api/v1/index failed");
    assert_eq!(status, 200, "unexpected status for index");

    validate_response_schema(&spec_json, "/api/v1/index", &response_json);
}

#[test]
fn featured_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/featured");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /api/v1/featured failed");
    assert_eq!(status, 200, "unexpected status for featured");

    validate_response_schema(&spec_json, "/api/v1/featured", &response_json);
}

#[test]
fn landing_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/landing");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /api/v1/landing failed");
    assert_eq!(status, 200, "unexpected status for landing");

    validate_response_schema(&spec_json, "/api/v1/landing", &response_json);
}

#[test]
fn modules_detail_404_matches_problem_details_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let missing_uuid = std::env::var("BARFORGE_API_MISSING_UUID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string());
    let url = format!("{base_url}/api/v1/modules/{missing_uuid}");

    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid} failed");
    let strict_data = std::env::var("BARFORGE_API_STRICT_DATA")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1");

    if strict_data {
        assert_eq!(status, 404, "expected 404 for missing module");
    }

    match status {
        404 => validate_response_schema_pointer(&spec_json, NOT_FOUND_SCHEMA_REF, &response_json),
        200 => validate_response_schema(&spec_json, "/api/v1/modules/{uuid}", &response_json),
        _ => panic!("unexpected status for missing module: {status}"),
    }
}

#[test]
fn categories_list_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/categories");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /api/v1/categories failed");
    assert_eq!(status, 200, "unexpected status for categories list");

    validate_response_schema(&spec_json, "/api/v1/categories", &response_json);
}

#[test]
fn related_modules_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let empty_related_uuid = std::env::var("BARFORGE_API_EMPTY_RELATED_UUID").ok();
    let uuid = empty_related_uuid
        .clone()
        .or_else(|| std::env::var("BARFORGE_API_RELATED_UUID").ok())
        .or_else(|| fetch_first_module_uuid(&base_url));

    let Some(uuid) = uuid else {
        eprintln!("Skipping related modules test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/related?limit=5");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid}/related failed");
    assert_eq!(status, 200, "unexpected status for related modules");

    validate_response_schema(&spec_json, "/api/v1/modules/{uuid}/related", &response_json);

    if let Some(expected_empty_uuid) = empty_related_uuid {
        if expected_empty_uuid == uuid {
            let modules = response_json
                .get("modules")
                .and_then(|value| value.as_array())
                .expect("related modules response missing modules array");
            assert!(modules.is_empty(), "expected no related modules");
        }
    }
}

#[test]
fn related_modules_invalid_limit_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping related invalid limit test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/related?limit=not-a-number");
    let (status, content_type, response_json) = send_empty_request_json_response("GET", &url)
        .expect("GET /api/v1/modules/{uuid}/related invalid limit failed");
    assert_eq!(status, 400, "expected 400 for invalid related limit");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn module_versions_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping module versions test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/versions");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid}/versions failed");
    assert_eq!(status, 200, "unexpected status for module versions");

    validate_response_schema(
        &spec_json,
        "/api/v1/modules/{uuid}/versions",
        &response_json,
    );
}

#[test]
fn module_reviews_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping module reviews test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid}/reviews failed");
    assert_eq!(status, 200, "unexpected status for module reviews");

    validate_response_schema(&spec_json, "/api/v1/modules/{uuid}/reviews", &response_json);
}

#[test]
fn module_download_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping module download test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/download");
    validate_empty_request(&request_validator, "POST", &url, None);

    let (status, content_type, body_len) = send_empty_request_text_response("POST", &url, None)
        .expect("POST /api/v1/modules/{uuid}/download failed");
    assert_eq!(status, 200, "unexpected status for module download");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/modules/{uuid}/download",
        "post",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected module download response body");
}

#[test]
fn module_upload_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping module upload test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping module upload test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }
    let Some((uuid, version, body, content_type)) = module_upload_payload() else {
        eprintln!(
            "Skipping module upload test; set BARFORGE_API_UPLOAD_UUID, BARFORGE_API_UPLOAD_VERSION, BARFORGE_API_UPLOAD_CONTENT_TYPE, BARFORGE_API_UPLOAD_PATH."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/versions/{version}/upload");
    validate_binary_request(
        &request_validator,
        "POST",
        &url,
        Some(&token),
        &content_type,
        body.len() as u64,
    );
    let (status, response_json) = send_binary_response(
        "POST",
        &url,
        &body,
        &content_type,
        Some(&token),
        Some(body.len() as u64),
    )
    .expect("POST /api/v1/modules/{uuid}/versions/{version}/upload failed");
    assert_eq!(status, 200, "unexpected status for module upload");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/modules/{uuid}/versions/{version}/upload",
        "post",
        &response_json,
    );
}

#[test]
fn module_publish_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping module publish test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping module publish test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some((uuid, version, payload)) = module_publish_payload() else {
        eprintln!(
            "Skipping module publish test; set BARFORGE_API_PUBLISH_UUID, BARFORGE_API_PUBLISH_VERSION, BARFORGE_API_PUBLISH_JSON to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/versions/{version}/publish");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, response_json) = send_json_response("POST", &url, &payload, Some(&token))
        .expect("POST /api/v1/modules/{uuid}/versions/{version}/publish failed");
    assert_eq!(status, 200, "unexpected status for module publish");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/modules/{uuid}/versions/{version}/publish",
        "post",
        &response_json,
    );
}

#[test]
fn module_publish_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping module publish invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping module publish invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some((uuid, version)) = module_publish_params() else {
        eprintln!(
            "Skipping module publish invalid payload test; set BARFORGE_API_PUBLISH_UUID and BARFORGE_API_PUBLISH_VERSION to enable."
        );
        return;
    };

    let payload = json!({ "changelog": 123 });
    let url = format!("{base_url}/api/v1/modules/{uuid}/versions/{version}/publish");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token)).expect(
            "POST /api/v1/modules/{uuid}/versions/{version}/publish invalid payload failed",
        );
    assert_eq!(status, 400, "expected 400 for invalid publish payload");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn module_screenshot_upload_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping screenshot upload test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping screenshot upload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some((uuid, body, content_type, alt_text)) = screenshot_upload_payload() else {
        eprintln!(
            "Skipping screenshot upload test; set BARFORGE_API_SCREENSHOT_UPLOAD_UUID, BARFORGE_API_SCREENSHOT_UPLOAD_PATH, BARFORGE_API_SCREENSHOT_UPLOAD_CONTENT_TYPE to enable."
        );
        return;
    };

    let url = format!(
        "{base_url}/api/v1/modules/{uuid}/screenshots{}",
        alt_text
            .as_ref()
            .map(|value| format!("?alt_text={value}"))
            .unwrap_or_default()
    );
    validate_binary_request(
        &request_validator,
        "POST",
        &url,
        Some(&token),
        &content_type,
        body.len() as u64,
    );
    let (status, response_json) = send_binary_response(
        "POST",
        &url,
        &body,
        &content_type,
        Some(&token),
        Some(body.len() as u64),
    )
    .expect("POST /api/v1/modules/{uuid}/screenshots failed");
    assert_eq!(status, 200, "unexpected status for screenshot upload");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/modules/{uuid}/screenshots",
        "post",
        &response_json,
    );
}

#[test]
fn module_screenshot_delete_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping screenshot delete test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!(
            "Skipping screenshot delete test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable."
        );
        return;
    }
    let Some((uuid, screenshot_id)) = screenshot_delete_params(&base_url) else {
        eprintln!(
            "Skipping screenshot delete test; set BARFORGE_API_SCREENSHOT_DELETE_UUID and BARFORGE_API_SCREENSHOT_DELETE_ID to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/screenshots/{screenshot_id}");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));
    let (status, response_json) =
        send_empty_request_json_response_with_auth("DELETE", &url, &token)
            .expect("DELETE /api/v1/modules/{uuid}/screenshots/{id} failed");
    assert_eq!(status, 200, "unexpected status for screenshot delete");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/modules/{uuid}/screenshots/{id}",
        "delete",
        &response_json,
    );
}

#[test]
fn module_review_create_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping review create test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping review create test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(uuid) = review_module_uuid(&base_url) else {
        eprintln!("Skipping review create test; no module UUID available.");
        return;
    };
    let Some(payload) = review_payload() else {
        eprintln!("Skipping review create test; set BARFORGE_API_REVIEW_PAYLOAD_JSON to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, content_type, body_len) =
        send_json_text_response("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/modules/{uuid}/reviews failed");
    assert_eq!(status, 200, "unexpected status for review create");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/modules/{uuid}/reviews",
        "post",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected review create response body");
}

#[test]
fn module_review_create_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping review create invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping review create invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(uuid) = review_module_uuid(&base_url) else {
        eprintln!("Skipping review create invalid payload test; no module UUID available.");
        return;
    };

    let payload = json!({});
    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/modules/{uuid}/reviews invalid payload failed");
    assert_eq!(
        status, 400,
        "expected 400 for invalid review create payload"
    );
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn module_review_update_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping review update test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping review update test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(uuid) = review_module_uuid(&base_url) else {
        eprintln!("Skipping review update test; no module UUID available.");
        return;
    };
    let Some(payload) = review_update_payload() else {
        eprintln!(
            "Skipping review update test; set BARFORGE_API_REVIEW_UPDATE_JSON or BARFORGE_API_REVIEW_PAYLOAD_JSON."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    validate_json_request(&request_validator, "PUT", &url, Some(&token), &payload);
    let (status, content_type, body_len) =
        send_json_text_response("PUT", &url, &payload, Some(&token))
            .expect("PUT /api/v1/modules/{uuid}/reviews failed");
    assert_eq!(status, 200, "unexpected status for review update");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/modules/{uuid}/reviews",
        "put",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected review update response body");
}

#[test]
fn module_review_update_invalid_payload_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping review update invalid payload test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping review update invalid payload test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(uuid) = review_module_uuid(&base_url) else {
        eprintln!("Skipping review update invalid payload test; no module UUID available.");
        return;
    };

    let payload = json!({});
    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    let (status, content_type, response_json) =
        send_json_response_with_content_type("PUT", &url, &payload, Some(&token))
            .expect("PUT /api/v1/modules/{uuid}/reviews invalid payload failed");
    assert_eq!(
        status, 400,
        "expected 400 for invalid review update payload"
    );
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn module_review_delete_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping review delete test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping review delete test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }
    let Some(uuid) = review_module_uuid(&base_url) else {
        eprintln!("Skipping review delete test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));
    let (status, content_type, body_len) =
        send_empty_request_text_response("DELETE", &url, Some(&token))
            .expect("DELETE /api/v1/modules/{uuid}/reviews failed");
    assert_eq!(status, 200, "unexpected status for review delete");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/modules/{uuid}/reviews",
        "delete",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected review delete response body");
}

#[test]
fn module_screenshots_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping module screenshots test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/screenshots");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid}/screenshots failed");
    assert_eq!(status, 200, "unexpected status for module screenshots");

    validate_response_schema(
        &spec_json,
        "/api/v1/modules/{uuid}/screenshots",
        &response_json,
    );
}

#[test]
fn screenshot_file_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some((uuid, filename)) = screenshot_path_params(&base_url) else {
        eprintln!("Skipping screenshot file test; no screenshot path available.");
        return;
    };

    let url = format!("{base_url}/screenshots/{uuid}/{filename}");
    validate_get_request(&request_validator, &url);

    let (status, content_type, content_length) =
        fetch_binary_response(&url).expect("GET /screenshots/{uuid}/{filename} failed");
    assert_eq!(status, 200, "unexpected status for screenshot file");
    assert_expected_content_type(
        &spec_json,
        "/screenshots/{uuid}/{filename}",
        "get",
        content_type.as_deref(),
    );
    assert_content_length(content_length);
}

#[test]
fn package_file_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some((uuid, version, filename)) = package_path_params(&base_url) else {
        eprintln!("Skipping package file test; no package path available.");
        return;
    };

    let url = format!("{base_url}/packages/{uuid}/{version}/{filename}");
    validate_get_request(&request_validator, &url);

    let (status, content_type, content_length) =
        fetch_binary_response(&url).expect("GET /packages/{uuid}/{version}/{filename} failed");
    assert_eq!(status, 200, "unexpected status for package file");
    assert_expected_content_type(
        &spec_json,
        "/packages/{uuid}/{version}/{filename}",
        "get",
        content_type.as_deref(),
    );
    assert_content_length(content_length);
}

#[test]
fn star_status_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(uuid) = fetch_star_module_uuid(&base_url) else {
        eprintln!("Skipping star status test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/modules/{uuid}/star failed");
    assert_eq!(status, 200, "unexpected status for star status");

    validate_response_schema(&spec_json, "/api/v1/modules/{uuid}/star", &response_json);
}

#[test]
fn stars_sync_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let Some(token) = auth_token() else {
        eprintln!("Skipping stars sync test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let Some(uuid) = fetch_star_module_uuid(&base_url) else {
        eprintln!("Skipping stars sync test; no module UUID available.");
        return;
    };

    let url = format!("{base_url}/api/v1/stars/sync");
    let payload = json!({ "uuids": [uuid] });
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, response_json) = send_json_response("POST", &url, &payload, Some(&token))
        .expect("POST /api/v1/stars/sync failed");
    assert_eq!(status, 200, "unexpected status for stars sync");

    validate_response_schema_for_method(&spec_json, "/api/v1/stars/sync", "post", &response_json);
}

#[test]
fn star_module_post_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping star POST test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping star POST test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(uuid) = fetch_star_module_uuid(&base_url) else {
        eprintln!("Skipping star POST test; no module UUID available.");
        return;
    };

    if fetch_star_status(&base_url, &uuid).unwrap_or(false) {
        let url = format!("{base_url}/api/v1/modules/{uuid}/star");
        let _ = send_empty_request_text_response("DELETE", &url, Some(&token));
    }

    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    if let Some(payload) = star_payload() {
        validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
        let (status, content_type, body_len) =
            send_json_text_response("POST", &url, &payload, Some(&token))
                .expect("POST /api/v1/modules/{uuid}/star failed");
        assert_eq!(status, 200, "unexpected status for star POST");
        assert_expected_content_type(
            &spec_json,
            "/api/v1/modules/{uuid}/star",
            "post",
            content_type.as_deref(),
        );
        assert!(body_len > 0, "expected star POST response body");
    } else {
        validate_empty_request(&request_validator, "POST", &url, Some(&token));
        let (status, content_type, body_len) =
            send_empty_request_text_response("POST", &url, Some(&token))
                .expect("POST /api/v1/modules/{uuid}/star failed");
        assert_eq!(status, 200, "unexpected status for star POST");
        assert_expected_content_type(
            &spec_json,
            "/api/v1/modules/{uuid}/star",
            "post",
            content_type.as_deref(),
        );
        assert!(body_len > 0, "expected star POST response body");
    }
}

#[test]
fn star_module_delete_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping star DELETE test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping star DELETE test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(uuid) = fetch_star_module_uuid(&base_url) else {
        eprintln!("Skipping star DELETE test; no module UUID available.");
        return;
    };

    if !fetch_star_status(&base_url, &uuid).unwrap_or(false) {
        let url = format!("{base_url}/api/v1/modules/{uuid}/star");
        let _ = send_empty_request_text_response("POST", &url, Some(&token));
    }

    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));
    let (status, content_type, body_len) =
        send_empty_request_text_response("DELETE", &url, Some(&token))
            .expect("DELETE /api/v1/modules/{uuid}/star failed");
    assert_eq!(status, 200, "unexpected status for star DELETE");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/modules/{uuid}/star",
        "delete",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected star DELETE response body");
}

#[test]
fn notifications_list_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping notifications list test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/notifications?limit=1&offset=0");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/notifications failed");
    assert_eq!(status, 200, "unexpected status for notifications list");

    validate_response_schema(&spec_json, "/api/v1/notifications", &response_json);
}

#[test]
fn notifications_preferences_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications preferences test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/preferences");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) = fetch_json_response_with_auth(&url, &token)
        .expect("GET /api/v1/notifications/preferences failed");
    assert_eq!(
        status, 200,
        "unexpected status for notifications preferences"
    );

    validate_response_schema(
        &spec_json,
        "/api/v1/notifications/preferences",
        &response_json,
    );
}

#[test]
fn notifications_preferences_update_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications preferences update test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping notifications preferences update test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }
    let Some(payload) = notification_preferences_payload() else {
        eprintln!(
            "Skipping notifications preferences update test; set BARFORGE_API_NOTIFICATION_PREFERENCES_JSON to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/preferences");
    validate_json_request(&request_validator, "PATCH", &url, Some(&token), &payload);

    let (status, content_type, response_json) =
        send_json_response_with_content_type("PATCH", &url, &payload, Some(&token))
            .expect("PATCH /api/v1/notifications/preferences failed");
    assert_eq!(
        status, 200,
        "unexpected status for notifications preferences update"
    );
    assert_expected_content_type(
        &spec_json,
        "/api/v1/notifications/preferences",
        "patch",
        content_type.as_deref(),
    );
    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/notifications/preferences",
        "patch",
        &response_json,
    );
}

#[test]
fn notifications_list_invalid_limit_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications invalid limit test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications?limit=not-a-number&offset=0");
    let (status, content_type, response_json) =
        send_empty_request_json_response_with_auth_content_type("GET", &url, &token)
            .expect("GET /api/v1/notifications invalid limit failed");
    assert_eq!(status, 400, "expected 400 for invalid notifications limit");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn notifications_list_invalid_offset_returns_problem_details() {
    let Some((base_url, spec_json, _request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications invalid offset test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications?limit=1&offset=not-a-number");
    let (status, content_type, response_json) =
        send_empty_request_json_response_with_auth_content_type("GET", &url, &token)
            .expect("GET /api/v1/notifications invalid offset failed");
    assert_eq!(status, 400, "expected 400 for invalid notifications offset");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

#[test]
fn notifications_unread_count_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications unread count test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/unread-count");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) = fetch_json_response_with_auth(&url, &token)
        .expect("GET /api/v1/notifications/unread-count failed");
    assert_eq!(
        status, 200,
        "unexpected status for notifications unread count"
    );

    validate_response_schema(
        &spec_json,
        "/api/v1/notifications/unread-count",
        &response_json,
    );
}

#[test]
fn notifications_mark_read_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping notifications mark read test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping notifications mark read test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }

    let Some(id) = fetch_first_notification_id(&base_url, &token) else {
        eprintln!("Skipping notifications mark read test; no notifications available.");
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/{id}/read");
    validate_empty_request(&request_validator, "PATCH", &url, Some(&token));

    let (status, response_json) = send_empty_request_json_response_with_auth("PATCH", &url, &token)
        .expect("PATCH /api/v1/notifications/{id}/read failed");
    assert_eq!(status, 200, "unexpected status for notifications mark read");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/notifications/{id}/read",
        "patch",
        &response_json,
    );
}

#[test]
fn notifications_mark_all_read_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!(
            "Skipping notifications mark all read test; set BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };
    if !allow_mutations() {
        eprintln!(
            "Skipping notifications mark all read test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable."
        );
        return;
    }

    let url = format!("{base_url}/api/v1/notifications/mark-all-read");
    validate_empty_request(&request_validator, "POST", &url, Some(&token));

    let (status, response_json) = send_empty_request_json_response_with_auth("POST", &url, &token)
        .expect("POST /api/v1/notifications/mark-all-read failed");
    assert_eq!(
        status, 200,
        "unexpected status for notifications mark all read"
    );

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/notifications/mark-all-read",
        "post",
        &response_json,
    );
}

#[test]
fn notifications_stream_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping notifications stream test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/stream");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, content_type, _) = fetch_binary_response_with_auth("GET", &url, &token)
        .expect("GET /api/v1/notifications/stream failed");
    assert_eq!(status, 200, "unexpected status for notifications stream");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/notifications/stream",
        "get",
        content_type.as_deref(),
    );
}

#[test]
fn notifications_announcements_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping announcements test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping announcements test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(payload) = announcement_payload() else {
        eprintln!(
            "Skipping announcements test; set BARFORGE_API_ANNOUNCEMENT_TITLE and BARFORGE_API_ANNOUNCEMENT_BODY to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/notifications/announcements");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, response_json) = send_json_response("POST", &url, &payload, Some(&token))
        .expect("POST /api/v1/notifications/announcements failed");
    assert_eq!(status, 200, "unexpected status for announcements");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/notifications/announcements",
        "post",
        &response_json,
    );
}

#[test]
fn admin_submission_approve_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = admin_token() else {
        eprintln!("Skipping admin approve test; set BARFORGE_API_ADMIN_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping admin approve test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }
    let Some(id) = submission_id() else {
        eprintln!("Skipping admin approve test; set BARFORGE_API_SUBMISSION_ID to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/submissions/{id}/approve");
    validate_empty_request(&request_validator, "POST", &url, Some(&token));
    let (status, content_type, body_len) =
        send_empty_request_text_response("POST", &url, Some(&token))
            .expect("POST /api/v1/admin/submissions/{id}/approve failed");
    assert_eq!(status, 200, "unexpected status for admin approve");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/admin/submissions/{id}/approve",
        "post",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected admin approve response body");
}

#[test]
fn admin_submission_reject_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = admin_token() else {
        eprintln!("Skipping admin reject test; set BARFORGE_API_ADMIN_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping admin reject test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }
    let Some(id) = submission_id() else {
        eprintln!("Skipping admin reject test; set BARFORGE_API_SUBMISSION_ID to enable.");
        return;
    };
    let Some(payload) = reject_payload() else {
        eprintln!("Skipping admin reject test; set BARFORGE_API_REJECT_REASON to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/submissions/{id}/reject");
    validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
    let (status, content_type, body_len) =
        send_json_text_response("POST", &url, &payload, Some(&token))
            .expect("POST /api/v1/admin/submissions/{id}/reject failed");
    assert_eq!(status, 200, "unexpected status for admin reject");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/admin/submissions/{id}/reject",
        "post",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected admin reject response body");
}

#[test]
fn admin_user_verify_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = admin_token() else {
        eprintln!("Skipping admin verify test; set BARFORGE_API_ADMIN_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping admin verify test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }
    let Some(id) = verify_user_id() else {
        eprintln!("Skipping admin verify test; set BARFORGE_API_VERIFY_USER_ID to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/users/{id}/verify");
    validate_empty_request(&request_validator, "POST", &url, Some(&token));
    let (status, response_json) = send_empty_request_json_response_with_auth("POST", &url, &token)
        .expect("POST /api/v1/admin/users/{id}/verify failed");
    assert_eq!(status, 200, "unexpected status for admin verify");

    validate_response_schema_for_method(
        &spec_json,
        "/api/v1/admin/users/{id}/verify",
        "post",
        &response_json,
    );
}

#[test]
fn admin_submissions_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = admin_token() else {
        eprintln!("Skipping admin submissions list test; set BARFORGE_API_ADMIN_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/submissions");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));
    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/admin/submissions failed");
    assert_eq!(status, 200, "unexpected status for admin submissions");

    validate_response_schema(&spec_json, "/api/v1/admin/submissions", &response_json);
}

#[test]
fn admin_stats_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = admin_token() else {
        eprintln!("Skipping admin stats test; set BARFORGE_API_ADMIN_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/stats");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));
    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/admin/stats failed");
    assert_eq!(status, 200, "unexpected status for admin stats");

    validate_response_schema(&spec_json, "/api/v1/admin/stats", &response_json);
}

#[test]
fn security_check_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some((uuid, version)) = security_check_params(&base_url) else {
        eprintln!(
            "Skipping security check test; set BARFORGE_API_SECURITY_UUID and BARFORGE_API_SECURITY_VERSION to enable."
        );
        return;
    };

    let url = format!("{base_url}/security/check?uuid={uuid}&version={version}");
    validate_get_request(&request_validator, &url);

    let (status, response_json) = fetch_json_response(&url).expect("GET /security/check failed");
    assert_eq!(status, 200, "unexpected status for security check");

    validate_response_schema(&spec_json, "/security/check", &response_json);
}

#[test]
fn health_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/health");
    validate_get_request(&request_validator, &url);

    let (status, content_type, body_len) =
        send_empty_request_text_response("GET", &url, None).expect("GET /api/v1/health failed");
    assert_eq!(status, 200, "unexpected status for health");
    assert_expected_content_type(&spec_json, "/api/v1/health", "get", content_type.as_deref());
    assert!(body_len > 0, "expected health response body");
}

#[test]
fn auth_verify_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/auth/verify");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/auth/verify failed");
    assert_eq!(status, 200, "unexpected status for auth verify");

    validate_response_schema(&spec_json, "/api/v1/auth/verify", &response_json);
}

#[test]
fn auth_sync_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping auth sync test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/auth/sync");
    validate_empty_request(&request_validator, "POST", &url, Some(&token));

    let (status, response_json) = send_empty_request_json_response_with_auth("POST", &url, &token)
        .expect("POST /api/v1/auth/sync failed");
    assert_eq!(status, 200, "unexpected status for auth sync");

    validate_response_schema_for_method(&spec_json, "/api/v1/auth/sync", "post", &response_json);
}

#[test]
fn user_profile_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let token = auth_token();
    let Some(username) = user_username(&base_url, token.as_deref()) else {
        eprintln!(
            "Skipping user profile test; set BARFORGE_API_USERNAME or BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/users/{username}");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/users/{username} failed");
    assert_eq!(status, 200, "unexpected status for user profile");

    validate_response_schema(&spec_json, "/api/v1/users/{username}", &response_json);
}

#[test]
fn user_modules_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let token = auth_token();
    let Some(username) = user_username(&base_url, token.as_deref()) else {
        eprintln!(
            "Skipping user modules test; set BARFORGE_API_USERNAME or BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/users/{username}/modules");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/users/{username}/modules failed");
    assert_eq!(status, 200, "unexpected status for user modules");

    validate_response_schema(
        &spec_json,
        "/api/v1/users/{username}/modules",
        &response_json,
    );
}

#[test]
fn user_collections_match_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let token = auth_token();
    let Some(username) = user_username(&base_url, token.as_deref()) else {
        eprintln!(
            "Skipping user collections test; set BARFORGE_API_USERNAME or BARFORGE_API_AUTH_TOKEN to enable."
        );
        return;
    };

    let url = format!("{base_url}/api/v1/users/{username}/collections");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/users/{username}/collections failed");
    assert_eq!(status, 200, "unexpected status for user collections");

    validate_response_schema(
        &spec_json,
        "/api/v1/users/{username}/collections",
        &response_json,
    );
}

#[test]
fn users_me_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping users/me test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/users/me");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/users/me failed");
    assert_eq!(status, 200, "unexpected status for users/me");

    validate_response_schema(&spec_json, "/api/v1/users/me", &response_json);
}

#[test]
fn users_me_update_matches_openapi_response() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping users/me update test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping users/me update test; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }
    let Some(payload) = profile_update_payload() else {
        eprintln!("Skipping users/me update test; set BARFORGE_API_PROFILE_UPDATE_JSON to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/users/me");
    validate_json_request(&request_validator, "PATCH", &url, Some(&token), &payload);

    let (status, content_type, body_len) =
        send_json_text_response("PATCH", &url, &payload, Some(&token))
            .expect("PATCH /api/v1/users/me failed");
    assert_eq!(status, 200, "unexpected status for users/me update");
    assert_expected_content_type(
        &spec_json,
        "/api/v1/users/me",
        "patch",
        content_type.as_deref(),
    );
    assert!(body_len > 0, "expected users/me update response body");
}

#[test]
fn users_me_delete_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping users/me delete test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    if !allow_destructive() {
        eprintln!("Skipping users/me delete test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable.");
        return;
    }

    let url = format!("{base_url}/api/v1/users/me");
    validate_empty_request(&request_validator, "DELETE", &url, Some(&token));

    let (status, response_json) =
        send_empty_request_json_response_with_auth("DELETE", &url, &token)
            .expect("DELETE /api/v1/users/me failed");
    assert_eq!(status, 200, "unexpected status for users/me delete");

    validate_response_schema_for_method(&spec_json, "/api/v1/users/me", "delete", &response_json);
}

#[test]
fn collections_list_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collections list test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/collections");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/collections failed");
    assert_eq!(status, 200, "unexpected status for collections list");

    validate_response_schema(&spec_json, "/api/v1/collections", &response_json);
}

#[test]
fn collection_detail_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping collection detail test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };
    let Some(id) = collection_id(&base_url, &token) else {
        eprintln!("Skipping collection detail test; set BARFORGE_API_COLLECTION_ID to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/collections/{id}");
    validate_get_request(&request_validator, &url);

    let (status, response_json) =
        fetch_json_response(&url).expect("GET /api/v1/collections/{id} failed");
    assert_eq!(status, 200, "unexpected status for collection detail");

    validate_response_schema(&spec_json, "/api/v1/collections/{id}", &response_json);
}

#[test]
fn modules_mine_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping modules/mine test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/modules/mine");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/modules/mine failed");
    assert_eq!(status, 200, "unexpected status for modules/mine");

    validate_response_schema(&spec_json, "/api/v1/modules/mine", &response_json);
}

#[test]
fn users_me_stars_matches_openapi_schema() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = auth_token() else {
        eprintln!("Skipping users/me/stars test; set BARFORGE_API_AUTH_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/users/me/stars");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));

    let (status, response_json) =
        fetch_json_response_with_auth(&url, &token).expect("GET /api/v1/users/me/stars failed");
    assert_eq!(status, 200, "unexpected status for users/me/stars");

    validate_response_schema(&spec_json, "/api/v1/users/me/stars", &response_json);
}

#[test]
fn auth_required_read_endpoints_reject_anonymous() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/api/v1/modules/mine");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/users/me");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/users/me/stars");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/notifications");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/notifications/unread-count");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/notifications/preferences");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/collections");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/admin/submissions");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);

    let url = format!("{base_url}/api/v1/admin/stats");
    validate_empty_request(&request_validator, "GET", &url, None);
    assert_unauthorized_empty_request(&spec_json, "GET", &url);
}

#[test]
fn auth_required_mutation_endpoints_reject_anonymous() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    if !allow_mutations() {
        eprintln!("Skipping auth mutation tests; set BARFORGE_API_ALLOW_MUTATIONS=1 to enable.");
        return;
    }

    let url = format!("{base_url}/api/v1/stars/sync");
    validate_empty_request(&request_validator, "POST", &url, None);
    assert_unauthorized_empty_request(&spec_json, "POST", &url);

    let url = format!("{base_url}/api/v1/notifications/mark-all-read");
    validate_empty_request(&request_validator, "POST", &url, None);
    assert_unauthorized_empty_request(&spec_json, "POST", &url);

    let Some(uuid) = fetch_first_module_uuid(&base_url) else {
        eprintln!("Skipping star auth tests; no module UUID available.");
        return;
    };
    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    validate_empty_request(&request_validator, "POST", &url, None);
    assert_unauthorized_empty_request(&spec_json, "POST", &url);

    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    validate_empty_request(&request_validator, "DELETE", &url, None);
    assert_unauthorized_empty_request(&spec_json, "DELETE", &url);

    if let Some(payload) = module_create_payload() {
        let url = format!("{base_url}/api/v1/modules");
        validate_json_request(&request_validator, "POST", &url, None, &payload);
        assert_unauthorized_json_request(&spec_json, "POST", &url, &payload);
    } else {
        eprintln!(
            "Skipping module create auth test; set BARFORGE_API_MODULE_CREATE_JSON to enable."
        );
    }

    if let Some((uuid, version, payload)) = module_publish_payload() {
        let url = format!("{base_url}/api/v1/modules/{uuid}/versions/{version}/publish");
        validate_json_request(&request_validator, "POST", &url, None, &payload);
        assert_unauthorized_json_request(&spec_json, "POST", &url, &payload);
    } else {
        eprintln!(
            "Skipping module publish auth test; set BARFORGE_API_PUBLISH_UUID, BARFORGE_API_PUBLISH_VERSION, BARFORGE_API_PUBLISH_JSON to enable."
        );
    }

    if let Some(uuid) = review_module_uuid(&base_url) {
        if let Some(payload) = review_payload() {
            let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
            validate_json_request(&request_validator, "POST", &url, None, &payload);
            assert_unauthorized_json_request(&spec_json, "POST", &url, &payload);
        } else {
            eprintln!(
                "Skipping review create auth test; set BARFORGE_API_REVIEW_PAYLOAD_JSON to enable."
            );
        }

        if let Some(payload) = review_update_payload() {
            let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
            validate_json_request(&request_validator, "PUT", &url, None, &payload);
            assert_unauthorized_json_request(&spec_json, "PUT", &url, &payload);
        } else {
            eprintln!(
                "Skipping review update auth test; set BARFORGE_API_REVIEW_UPDATE_JSON or BARFORGE_API_REVIEW_PAYLOAD_JSON."
            );
        }
    } else {
        eprintln!("Skipping review auth tests; no module UUID available.");
    }

    if let Some(payload) = profile_update_payload() {
        let url = format!("{base_url}/api/v1/users/me");
        validate_json_request(&request_validator, "PATCH", &url, None, &payload);
        assert_unauthorized_json_request(&spec_json, "PATCH", &url, &payload);
    } else {
        eprintln!(
            "Skipping users/me update auth test; set BARFORGE_API_PROFILE_UPDATE_JSON to enable."
        );
    }

    if let Some(payload) = announcement_payload() {
        let url = format!("{base_url}/api/v1/notifications/announcements");
        validate_json_request(&request_validator, "POST", &url, None, &payload);
        assert_unauthorized_json_request(&spec_json, "POST", &url, &payload);
    } else {
        eprintln!(
            "Skipping announcements auth test; set BARFORGE_API_ANNOUNCEMENT_TITLE and BARFORGE_API_ANNOUNCEMENT_BODY to enable."
        );
    }
}

#[test]
fn auth_required_destructive_endpoints_reject_anonymous() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    if !allow_destructive() {
        eprintln!(
            "Skipping auth destructive tests; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable."
        );
        return;
    }

    if let Some((uuid, version, body, content_type)) = module_upload_payload() {
        let url = format!("{base_url}/api/v1/modules/{uuid}/versions/{version}/upload");
        validate_binary_request(
            &request_validator,
            "POST",
            &url,
            None,
            &content_type,
            body.len() as u64,
        );
        assert_unauthorized_binary_request(
            &spec_json,
            "POST",
            &url,
            &body,
            &content_type,
            Some(body.len() as u64),
        );
    } else {
        eprintln!(
            "Skipping module upload auth test; set BARFORGE_API_UPLOAD_UUID, BARFORGE_API_UPLOAD_VERSION, BARFORGE_API_UPLOAD_CONTENT_TYPE, BARFORGE_API_UPLOAD_PATH."
        );
    }

    if let Some((uuid, body, content_type, alt_text)) = screenshot_upload_payload() {
        let url = format!(
            "{base_url}/api/v1/modules/{uuid}/screenshots{}",
            alt_text
                .as_ref()
                .map(|value| format!("?alt_text={value}"))
                .unwrap_or_default()
        );
        validate_binary_request(
            &request_validator,
            "POST",
            &url,
            None,
            &content_type,
            body.len() as u64,
        );
        assert_unauthorized_binary_request(
            &spec_json,
            "POST",
            &url,
            &body,
            &content_type,
            Some(body.len() as u64),
        );
    } else {
        eprintln!(
            "Skipping screenshot upload auth test; set BARFORGE_API_SCREENSHOT_UPLOAD_UUID, BARFORGE_API_SCREENSHOT_UPLOAD_PATH, BARFORGE_API_SCREENSHOT_UPLOAD_CONTENT_TYPE to enable."
        );
    }

    if let Some((uuid, screenshot_id)) = screenshot_delete_params(&base_url) {
        let url = format!("{base_url}/api/v1/modules/{uuid}/screenshots/{screenshot_id}");
        validate_empty_request(&request_validator, "DELETE", &url, None);
        assert_unauthorized_empty_request(&spec_json, "DELETE", &url);
    } else {
        eprintln!(
            "Skipping screenshot delete auth test; set BARFORGE_API_SCREENSHOT_DELETE_UUID and BARFORGE_API_SCREENSHOT_DELETE_ID to enable."
        );
    }

    if let Some(uuid) = review_module_uuid(&base_url) {
        let url = format!("{base_url}/api/v1/modules/{uuid}/reviews");
        validate_empty_request(&request_validator, "DELETE", &url, None);
        assert_unauthorized_empty_request(&spec_json, "DELETE", &url);
    } else {
        eprintln!("Skipping review delete auth test; no module UUID available.");
    }

    let url = format!("{base_url}/api/v1/users/me");
    validate_empty_request(&request_validator, "DELETE", &url, None);
    assert_unauthorized_empty_request(&spec_json, "DELETE", &url);
}

#[test]
fn admin_endpoints_reject_non_admin() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = non_admin_token() else {
        eprintln!("Skipping admin forbidden test; set BARFORGE_API_NON_ADMIN_TOKEN to enable.");
        return;
    };

    let url = format!("{base_url}/api/v1/admin/submissions");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));
    let (status, content_type, response_json) =
        send_empty_request_json_response_with_auth_content_type("GET", &url, &token)
            .expect("GET /api/v1/admin/submissions with non-admin failed");
    assert_forbidden_status(status, "GET", &url);
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);

    let url = format!("{base_url}/api/v1/admin/stats");
    validate_empty_request(&request_validator, "GET", &url, Some(&token));
    let (status, content_type, response_json) =
        send_empty_request_json_response_with_auth_content_type("GET", &url, &token)
            .expect("GET /api/v1/admin/stats with non-admin failed");
    assert_forbidden_status(status, "GET", &url);
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
}

#[test]
fn admin_mutation_endpoints_reject_non_admin() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };
    let Some(token) = non_admin_token() else {
        eprintln!(
            "Skipping admin mutation forbidden test; set BARFORGE_API_NON_ADMIN_TOKEN to enable."
        );
        return;
    };
    if !allow_destructive() {
        eprintln!(
            "Skipping admin mutation forbidden test; set BARFORGE_API_ALLOW_DESTRUCTIVE=1 to enable."
        );
        return;
    }

    if let Some(id) = submission_id() {
        let url = format!("{base_url}/api/v1/admin/submissions/{id}/approve");
        validate_empty_request(&request_validator, "POST", &url, Some(&token));
        let (status, content_type, response_json) =
            send_empty_request_json_response_with_auth_content_type("POST", &url, &token)
                .expect("POST /api/v1/admin/submissions/{id}/approve with non-admin failed");
        assert_forbidden_status(status, "POST", &url);
        assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    } else {
        eprintln!(
            "Skipping admin approve forbidden test; set BARFORGE_API_SUBMISSION_ID to enable."
        );
    }

    if let (Some(id), Some(payload)) = (submission_id(), reject_payload()) {
        let url = format!("{base_url}/api/v1/admin/submissions/{id}/reject");
        validate_json_request(&request_validator, "POST", &url, Some(&token), &payload);
        let (status, content_type, response_json) =
            send_json_response_with_content_type("POST", &url, &payload, Some(&token))
                .expect("POST /api/v1/admin/submissions/{id}/reject with non-admin failed");
        assert_forbidden_status(status, "POST", &url);
        assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    } else {
        eprintln!(
            "Skipping admin reject forbidden test; set BARFORGE_API_SUBMISSION_ID and BARFORGE_API_REJECT_REASON to enable."
        );
    }

    if let Some(id) = verify_user_id() {
        let url = format!("{base_url}/api/v1/admin/users/{id}/verify");
        validate_empty_request(&request_validator, "POST", &url, Some(&token));
        let (status, content_type, response_json) =
            send_empty_request_json_response_with_auth_content_type("POST", &url, &token)
                .expect("POST /api/v1/admin/users/{id}/verify with non-admin failed");
        assert_forbidden_status(status, "POST", &url);
        assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    } else {
        eprintln!(
            "Skipping admin verify forbidden test; set BARFORGE_API_VERIFY_USER_ID to enable."
        );
    }
}

#[test]
fn security_check_missing_params_returns_problem_details() {
    let Some((base_url, spec_json, request_validator)) = live_context() else {
        return;
    };

    let url = format!("{base_url}/security/check");
    validate_get_request(&request_validator, &url);

    let (status, content_type, response_json) =
        send_empty_request_json_response("GET", &url).expect("GET /security/check failed");
    assert_eq!(status, 400, "expected 400 for missing security params");
    assert_problem_details(&spec_json, status, content_type.as_deref(), &response_json);
    if strict_errors() {
        assert_problem_code(&response_json);
    }
}

fn live_api_base_url() -> Option<String> {
    let live = std::env::var("BARFORGE_API_LIVE").ok();
    if live.as_deref() != Some("1") {
        eprintln!("Skipping live API test; set BARFORGE_API_LIVE=1 to enable.");
        return None;
    }

    let base_url = std::env::var("BARFORGE_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.barforge.dev".to_string());

    Some(base_url.trim_end_matches('/').to_string())
}

fn live_context() -> Option<(String, Value, OpenApiPayloadValidator)> {
    let base_url = live_api_base_url()?;
    let openapi_path = openapi_path();
    let spec_json = barforge_contracts::openapi::load_openapi_json(&openapi_path)
        .expect("failed to load openapi spec as json");
    let request_validator =
        OpenApiPayloadValidator::new(spec_json.clone()).expect("failed to build request validator");

    Some((base_url, spec_json, request_validator))
}

fn openapi_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../../barforge-registry-api/docs/openapi.yaml")
}

fn validate_get_request(request_validator: &OpenApiPayloadValidator, url: &str) {
    validate_empty_request(request_validator, "GET", url, None);
}

fn validate_empty_request(
    request_validator: &OpenApiPayloadValidator,
    method: &str,
    url: &str,
    auth_token: Option<&str>,
) {
    let mut builder = Request::builder().method(method).uri(url);
    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(()).expect("failed to build request");

    request_validator
        .validate_request(&request, None)
        .expect("request should match openapi schema");
}

fn validate_json_request(
    request_validator: &OpenApiPayloadValidator,
    method: &str,
    url: &str,
    auth_token: Option<&str>,
    payload: &Value,
) {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json");
    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder
        .body(payload.clone())
        .expect("failed to build request");

    request_validator
        .validate_request(&request, None)
        .expect("request should match openapi schema");
}

fn validate_binary_request(
    request_validator: &OpenApiPayloadValidator,
    method: &str,
    url: &str,
    auth_token: Option<&str>,
    content_type: &str,
    content_length: u64,
) {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", content_type)
        .header("Content-Length", content_length.to_string());
    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(()).expect("failed to build request");

    request_validator
        .validate_request(&request, None)
        .expect("request should match openapi schema");
}

fn fetch_json_response(url: &str) -> Result<(u16, Value), ureq::Error> {
    let response = send_empty_request("GET", url, None)?;

    let status = response.status().as_u16();
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, response_json))
}

fn fetch_json_response_with_auth(url: &str, auth_token: &str) -> Result<(u16, Value), ureq::Error> {
    let response = send_empty_request("GET", url, Some(auth_token))?;
    let status = response.status().as_u16();
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, response_json))
}

fn send_empty_request_json_response_with_auth(
    method: &str,
    url: &str,
    auth_token: &str,
) -> Result<(u16, Value), ureq::Error> {
    let response = send_empty_request(method, url, Some(auth_token))?;
    let status = response.status().as_u16();
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, response_json))
}

fn send_empty_request_json_response_with_auth_content_type(
    method: &str,
    url: &str,
    auth_token: &str,
) -> Result<(u16, Option<String>, Value), ureq::Error> {
    let response = send_empty_request(method, url, Some(auth_token))?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, content_type, response_json))
}

fn send_empty_request_json_response(
    method: &str,
    url: &str,
) -> Result<(u16, Option<String>, Value), ureq::Error> {
    let response = send_empty_request(method, url, None)?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, content_type, response_json))
}

fn send_json_response(
    method: &str,
    url: &str,
    payload: &Value,
    auth_token: Option<&str>,
) -> Result<(u16, Value), ureq::Error> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = builder
        .body(payload.to_string())
        .expect("failed to build request");

    let response = request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()?;
    let status = response.status().as_u16();
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, response_json))
}

fn send_json_response_with_content_type(
    method: &str,
    url: &str,
    payload: &Value,
    auth_token: Option<&str>,
) -> Result<(u16, Option<String>, Value), ureq::Error> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = builder
        .body(payload.to_string())
        .expect("failed to build request");

    let response = request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, content_type, response_json))
}

fn fetch_binary_response(url: &str) -> Result<(u16, Option<String>, Option<u64>), ureq::Error> {
    let response = send_empty_request("GET", url, None)?;

    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let content_length = response
        .headers()
        .get("Content-Length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());

    Ok((status, content_type, content_length))
}

fn fetch_binary_response_with_auth(
    method: &str,
    url: &str,
    auth_token: &str,
) -> Result<(u16, Option<String>, Option<u64>), ureq::Error> {
    let response = send_empty_request(method, url, Some(auth_token))?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let content_length = response
        .headers()
        .get("Content-Length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());

    Ok((status, content_type, content_length))
}

fn send_json_text_response(
    method: &str,
    url: &str,
    payload: &Value,
    auth_token: Option<&str>,
) -> Result<(u16, Option<String>, usize), ureq::Error> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = builder
        .body(payload.to_string())
        .expect("failed to build request");
    let response = request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()?;

    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let text = response
        .into_body()
        .read_to_string()
        .expect("failed to read response body");

    Ok((status, content_type, text.len()))
}

fn send_empty_request_text_response(
    method: &str,
    url: &str,
    auth_token: Option<&str>,
) -> Result<(u16, Option<String>, usize), ureq::Error> {
    let response = send_empty_request(method, url, auth_token)?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let text = response
        .into_body()
        .read_to_string()
        .expect("failed to read response body");

    Ok((status, content_type, text.len()))
}

fn send_binary_response(
    method: &str,
    url: &str,
    body: &[u8],
    content_type: &str,
    auth_token: Option<&str>,
    content_length: Option<u64>,
) -> Result<(u16, Value), ureq::Error> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", content_type);

    if let Some(length) = content_length {
        builder = builder.header("Content-Length", length.to_string());
    }

    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = builder
        .body(body.to_vec())
        .expect("failed to build request");

    let response = request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()?;
    let status = response.status().as_u16();
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, response_json))
}

fn send_binary_response_with_content_type(
    method: &str,
    url: &str,
    body: &[u8],
    content_type: &str,
    auth_token: Option<&str>,
    content_length: Option<u64>,
) -> Result<(u16, Option<String>, Value), ureq::Error> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", content_type);

    if let Some(length) = content_length {
        builder = builder.header("Content-Length", length.to_string());
    }

    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request = builder
        .body(body.to_vec())
        .expect("failed to build request");

    let response = request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let mut body = response.into_body();
    let response_json: Value = body.read_json()?;

    Ok((status, content_type, response_json))
}

fn send_empty_request(
    method: &str,
    url: &str,
    auth_token: Option<&str>,
) -> Result<http::Response<ureq::Body>, ureq::Error> {
    let mut builder = Request::builder().method(method).uri(url);
    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(()).expect("failed to build request");

    request
        .with_default_agent()
        .configure()
        .http_status_as_error(false)
        .run()
}

fn fetch_first_module_summary(base_url: &str) -> Option<(String, Option<String>)> {
    let url = format!("{base_url}/api/v1/modules?limit=1");
    let (status, response_json) = fetch_json_response(&url).ok()?;
    if status != 200 {
        return None;
    }

    let module = response_json
        .get("modules")
        .and_then(|value| value.as_array())
        .and_then(|modules| modules.first())?;
    let uuid = module
        .get("uuid")
        .and_then(|uuid| uuid.as_str())?
        .to_string();
    let version = module
        .get("version")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    Some((uuid, version))
}

fn fetch_first_module_uuid(base_url: &str) -> Option<String> {
    fetch_first_module_summary(base_url).map(|(uuid, _)| uuid)
}

fn fetch_first_module_version(base_url: &str) -> Option<String> {
    fetch_first_module_summary(base_url).and_then(|(_, version)| version)
}

fn fetch_star_module_uuid(base_url: &str) -> Option<String> {
    std::env::var("BARFORGE_API_STAR_UUID")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| fetch_first_module_uuid(base_url))
}

fn fetch_star_status(base_url: &str, uuid: &str) -> Option<bool> {
    let url = format!("{base_url}/api/v1/modules/{uuid}/star");
    let (status, response_json) = fetch_json_response(&url).ok()?;
    if status != 200 {
        return None;
    }

    response_json
        .get("starred")
        .and_then(|value| value.as_bool())
}

fn fetch_first_notification_id(base_url: &str, auth_token: &str) -> Option<i64> {
    let url = format!("{base_url}/api/v1/notifications?limit=1&offset=0");
    let (status, response_json) = fetch_json_response_with_auth(&url, auth_token).ok()?;
    if status != 200 {
        return None;
    }

    response_json
        .get("notifications")
        .and_then(|value| value.as_array())
        .and_then(|notifications| notifications.first())
        .and_then(|notification| notification.get("id"))
        .and_then(|id| id.as_i64())
}

fn fetch_username_from_me(base_url: &str, auth_token: &str) -> Option<String> {
    let url = format!("{base_url}/api/v1/users/me");
    let (status, response_json) = fetch_json_response_with_auth(&url, auth_token).ok()?;
    if status != 200 {
        return None;
    }

    response_json
        .get("username")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn user_username(base_url: &str, auth_token: Option<&str>) -> Option<String> {
    std::env::var("BARFORGE_API_USERNAME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| auth_token.and_then(|token| fetch_username_from_me(base_url, token)))
}

fn fetch_first_collection_id(base_url: &str, auth_token: &str) -> Option<i64> {
    let url = format!("{base_url}/api/v1/collections");
    let (status, response_json) = fetch_json_response_with_auth(&url, auth_token).ok()?;
    if status != 200 {
        return None;
    }

    response_json
        .get("collections")
        .and_then(|value| value.as_array())
        .and_then(|collections| collections.first())
        .and_then(|collection| collection.get("id"))
        .and_then(|id| id.as_i64())
}

fn collection_id(base_url: &str, auth_token: &str) -> Option<i64> {
    std::env::var("BARFORGE_API_COLLECTION_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| fetch_first_collection_id(base_url, auth_token))
}

fn collection_delete_id() -> Option<i64> {
    std::env::var("BARFORGE_API_COLLECTION_DELETE_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
}

fn collection_add_id(base_url: &str, auth_token: &str) -> Option<i64> {
    std::env::var("BARFORGE_API_COLLECTION_ADD_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| collection_id(base_url, auth_token))
}

fn collection_remove_params() -> Option<(i64, String)> {
    let id = std::env::var("BARFORGE_API_COLLECTION_REMOVE_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())?;
    let uuid = std::env::var("BARFORGE_API_COLLECTION_REMOVE_UUID")
        .ok()
        .filter(|value| !value.is_empty())?;

    Some((id, uuid))
}

fn screenshot_path_params(base_url: &str) -> Option<(String, String)> {
    if let (Ok(uuid), Ok(filename)) = (
        std::env::var("BARFORGE_API_SCREENSHOT_UUID"),
        std::env::var("BARFORGE_API_SCREENSHOT_FILENAME"),
    ) {
        if !uuid.is_empty() && !filename.is_empty() {
            return Some((uuid, filename));
        }
    }

    let uuid = fetch_first_module_uuid(base_url)?;
    let url = format!("{base_url}/api/v1/modules/{uuid}/screenshots");
    let (status, response_json) = fetch_json_response(&url).ok()?;
    if status != 200 {
        return None;
    }

    let r2_key = response_json
        .get("screenshots")
        .and_then(|value| value.as_array())
        .and_then(|screenshots| screenshots.first())
        .and_then(|screenshot| screenshot.get("r2_key"))
        .and_then(|value| value.as_str())?;
    let filename = r2_key.split('/').last()?.to_string();
    if filename.is_empty() {
        return None;
    }

    Some((uuid, filename))
}

fn package_path_params(base_url: &str) -> Option<(String, String, String)> {
    if let (Ok(uuid), Ok(version), Ok(filename)) = (
        std::env::var("BARFORGE_API_PACKAGE_UUID"),
        std::env::var("BARFORGE_API_PACKAGE_VERSION"),
        std::env::var("BARFORGE_API_PACKAGE_FILENAME"),
    ) {
        if !uuid.is_empty() && !version.is_empty() && !filename.is_empty() {
            return Some((uuid, version, filename));
        }
    }

    let filename = std::env::var("BARFORGE_API_PACKAGE_FILENAME").ok()?;
    if filename.is_empty() {
        return None;
    }

    let uuid = std::env::var("BARFORGE_API_PACKAGE_UUID")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| fetch_first_module_uuid(base_url))?;
    let version = std::env::var("BARFORGE_API_PACKAGE_VERSION")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| fetch_first_module_version(base_url))?;

    Some((uuid, version, filename))
}

fn auth_token() -> Option<String> {
    std::env::var("BARFORGE_API_AUTH_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
}

fn non_admin_token() -> Option<String> {
    std::env::var("BARFORGE_API_NON_ADMIN_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
}

fn admin_token() -> Option<String> {
    std::env::var("BARFORGE_API_ADMIN_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
}

fn allow_mutations() -> bool {
    std::env::var("BARFORGE_API_ALLOW_MUTATIONS")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1")
}

fn allow_destructive() -> bool {
    std::env::var("BARFORGE_API_ALLOW_DESTRUCTIVE")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1")
}

fn strict_errors() -> bool {
    std::env::var("BARFORGE_API_STRICT_ERRORS")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1")
}

fn submission_id() -> Option<i64> {
    std::env::var("BARFORGE_API_SUBMISSION_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
}

fn verify_user_id() -> Option<i64> {
    std::env::var("BARFORGE_API_VERIFY_USER_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
}

fn announcement_payload() -> Option<Value> {
    let title = std::env::var("BARFORGE_API_ANNOUNCEMENT_TITLE")
        .ok()
        .filter(|value| !value.is_empty())?;
    let body = std::env::var("BARFORGE_API_ANNOUNCEMENT_BODY")
        .ok()
        .filter(|value| !value.is_empty())?;
    let action_url = std::env::var("BARFORGE_API_ANNOUNCEMENT_ACTION_URL")
        .ok()
        .filter(|value| !value.is_empty());

    Some(json!({
        "title": title,
        "body": body,
        "action_url": action_url
    }))
}

fn profile_update_payload() -> Option<Value> {
    let raw = std::env::var("BARFORGE_API_PROFILE_UPDATE_JSON")
        .ok()
        .filter(|value| !value.is_empty())?;

    serde_json::from_str(&raw).ok()
}

fn notification_preferences_payload() -> Option<Value> {
    let raw = std::env::var("BARFORGE_API_NOTIFICATION_PREFERENCES_JSON")
        .ok()
        .filter(|value| !value.is_empty())?;

    serde_json::from_str(&raw).ok()
}

fn reject_payload() -> Option<Value> {
    let reason = std::env::var("BARFORGE_API_REJECT_REASON")
        .ok()
        .filter(|value| !value.is_empty())?;

    Some(json!({ "reason": reason }))
}

fn review_module_uuid(base_url: &str) -> Option<String> {
    std::env::var("BARFORGE_API_REVIEW_UUID")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| fetch_first_module_uuid(base_url))
}

fn review_payload() -> Option<Value> {
    let raw = std::env::var("BARFORGE_API_REVIEW_PAYLOAD_JSON")
        .ok()
        .filter(|value| !value.is_empty())?;

    serde_json::from_str(&raw).ok()
}

fn review_update_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_REVIEW_UPDATE_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .or_else(review_payload)
}

fn star_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_STAR_PAYLOAD_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn module_create_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_MODULE_CREATE_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn collection_create_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_COLLECTION_CREATE_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn collection_update_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_COLLECTION_UPDATE_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn collection_add_module_payload() -> Option<Value> {
    std::env::var("BARFORGE_API_COLLECTION_ADD_MODULE_JSON")
        .ok()
        .filter(|value| !value.is_empty())
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn module_upload_payload() -> Option<(String, String, Vec<u8>, String)> {
    let uuid = std::env::var("BARFORGE_API_UPLOAD_UUID")
        .ok()
        .filter(|value| !value.is_empty())?;
    let version = std::env::var("BARFORGE_API_UPLOAD_VERSION")
        .ok()
        .filter(|value| !value.is_empty())?;
    let content_type = std::env::var("BARFORGE_API_UPLOAD_CONTENT_TYPE")
        .ok()
        .filter(|value| !value.is_empty())?;
    let path = std::env::var("BARFORGE_API_UPLOAD_PATH")
        .ok()
        .filter(|value| !value.is_empty())?;
    let body = std::fs::read(path).ok()?;

    Some((uuid, version, body, content_type))
}

fn module_publish_params() -> Option<(String, String)> {
    let uuid = std::env::var("BARFORGE_API_PUBLISH_UUID")
        .ok()
        .filter(|value| !value.is_empty())?;
    let version = std::env::var("BARFORGE_API_PUBLISH_VERSION")
        .ok()
        .filter(|value| !value.is_empty())?;

    Some((uuid, version))
}

fn module_publish_payload() -> Option<(String, String, Value)> {
    let (uuid, version) = module_publish_params()?;
    let raw = std::env::var("BARFORGE_API_PUBLISH_JSON")
        .ok()
        .filter(|value| !value.is_empty())?;
    let payload = serde_json::from_str(&raw).ok()?;

    Some((uuid, version, payload))
}

fn screenshot_upload_payload() -> Option<(String, Vec<u8>, String, Option<String>)> {
    let uuid = std::env::var("BARFORGE_API_SCREENSHOT_UPLOAD_UUID")
        .ok()
        .filter(|value| !value.is_empty())?;
    let content_type = std::env::var("BARFORGE_API_SCREENSHOT_UPLOAD_CONTENT_TYPE")
        .ok()
        .filter(|value| !value.is_empty())?;
    let path = std::env::var("BARFORGE_API_SCREENSHOT_UPLOAD_PATH")
        .ok()
        .filter(|value| !value.is_empty())?;
    let body = std::fs::read(path).ok()?;
    let alt_text = std::env::var("BARFORGE_API_SCREENSHOT_UPLOAD_ALT_TEXT")
        .ok()
        .filter(|value| !value.is_empty());

    Some((uuid, body, content_type, alt_text))
}

fn screenshot_delete_params(base_url: &str) -> Option<(String, i64)> {
    if let (Ok(uuid), Ok(id)) = (
        std::env::var("BARFORGE_API_SCREENSHOT_DELETE_UUID"),
        std::env::var("BARFORGE_API_SCREENSHOT_DELETE_ID"),
    ) {
        if !uuid.is_empty() {
            if let Ok(id) = id.parse::<i64>() {
                return Some((uuid, id));
            }
        }
    }

    let uuid = fetch_first_module_uuid(base_url)?;
    let url = format!("{base_url}/api/v1/modules/{uuid}/screenshots");
    let (status, response_json) = fetch_json_response(&url).ok()?;
    if status != 200 {
        return None;
    }

    let screenshot_id = response_json
        .get("screenshots")
        .and_then(|value| value.as_array())
        .and_then(|screenshots| screenshots.first())
        .and_then(|screenshot| screenshot.get("id"))
        .and_then(|id| id.as_i64())?;

    Some((uuid, screenshot_id))
}

fn security_check_params(base_url: &str) -> Option<(String, String)> {
    if let (Ok(uuid), Ok(version)) = (
        std::env::var("BARFORGE_API_SECURITY_UUID"),
        std::env::var("BARFORGE_API_SECURITY_VERSION"),
    ) {
        if !uuid.is_empty() && !version.is_empty() {
            return Some((uuid, version));
        }
    }

    let (uuid, version) = fetch_first_module_summary(base_url)?;
    let Some(version) = version else {
        return None;
    };

    Some((uuid, version))
}

fn validate_response_schema(spec_json: &Value, endpoint: &str, instance: &Value) {
    validate_response_schema_for_method(spec_json, endpoint, "get", instance);
}

fn validate_response_schema_for_method(
    spec_json: &Value,
    endpoint: &str,
    method: &str,
    instance: &Value,
) {
    let schema_ref = format!(
        "@@root#/paths/{}/{}/responses/200/content/{}/schema",
        json_pointer_escape(endpoint),
        method,
        json_pointer_escape("application/json"),
    );
    validate_response_schema_pointer(spec_json, &schema_ref, instance);
}

fn validate_response_schema_pointer(spec_json: &Value, schema_ref: &str, instance: &Value) {
    let schema = json!({ "$ref": schema_ref });

    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_resource("@@root", Resource::from_contents(spec_json.clone()))
        .build(&schema)
        .expect("failed to build schema validator");

    if let Err(error) = validator.validate(instance) {
        panic!("response does not match schema: {error}");
    }
}

fn json_pointer_escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn expected_content_types(spec_json: &Value, endpoint: &str, method: &str) -> Vec<String> {
    let pointer = format!(
        "/paths/{}/{}/responses/200/content",
        json_pointer_escape(endpoint),
        method,
    );
    spec_json
        .pointer(&pointer)
        .and_then(|value| value.as_object())
        .map(|content| content.keys().cloned().collect())
        .unwrap_or_default()
}

fn assert_expected_content_type(
    spec_json: &Value,
    endpoint: &str,
    method: &str,
    content_type: Option<&str>,
) {
    let expected = expected_content_types(spec_json, endpoint, method);
    if expected.is_empty() {
        panic!("missing content types in openapi spec for {endpoint}");
    }

    let Some(content_type) = content_type else {
        panic!("missing Content-Type header for {endpoint}");
    };
    let normalized = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();

    if !expected.iter().any(|value| value == normalized) {
        panic!(
            "unexpected Content-Type for {endpoint}: {normalized}, expected one of {expected:?}"
        );
    }
}

fn assert_content_length(content_length: Option<u64>) {
    match content_length {
        Some(length) if length > 0 => {}
        _ => panic!("response body is empty"),
    }
}

fn assert_unauthorized_empty_request(spec_json: &Value, method: &str, url: &str) {
    let (status, content_type, response_json) =
        send_empty_request_json_response(method, url).expect("unauthorized request failed");
    assert_auth_error_status(status, method, url);
    assert_problem_details(spec_json, status, content_type.as_deref(), &response_json);
}

fn assert_unauthorized_json_request(spec_json: &Value, method: &str, url: &str, payload: &Value) {
    let (status, content_type, response_json) =
        send_json_response_with_content_type(method, url, payload, None)
            .expect("unauthorized json request failed");
    assert_auth_error_status(status, method, url);
    assert_problem_details(spec_json, status, content_type.as_deref(), &response_json);
}

fn assert_unauthorized_binary_request(
    spec_json: &Value,
    method: &str,
    url: &str,
    body: &[u8],
    content_type: &str,
    content_length: Option<u64>,
) {
    let (status, content_type, response_json) = send_binary_response_with_content_type(
        method,
        url,
        body,
        content_type,
        None,
        content_length,
    )
    .expect("unauthorized binary request failed");
    assert_auth_error_status(status, method, url);
    assert_problem_details(spec_json, status, content_type.as_deref(), &response_json);
}

fn assert_auth_error_status(status: u16, method: &str, url: &str) {
    if strict_errors() {
        assert_eq!(status, 401, "expected 401 for {method} {url}, got {status}");
    } else if status != 401 && status != 403 {
        panic!("expected 401 or 403 for {method} {url}, got {status}");
    }
}

fn assert_forbidden_status(status: u16, method: &str, url: &str) {
    if strict_errors() {
        assert_eq!(status, 403, "expected 403 for {method} {url}, got {status}");
    } else if status != 401 && status != 403 {
        panic!("expected 401 or 403 for {method} {url}, got {status}");
    }
}

fn assert_problem_details(
    spec_json: &Value,
    status: u16,
    content_type: Option<&str>,
    response_json: &Value,
) {
    assert_problem_content_type(content_type);
    validate_response_schema_pointer(spec_json, PROBLEM_DETAILS_SCHEMA_REF, response_json);
    assert_problem_status(response_json, status);
}

fn assert_problem_content_type(content_type: Option<&str>) {
    let Some(content_type) = content_type else {
        panic!("missing Content-Type header for problem response");
    };
    let normalized = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    assert_eq!(
        normalized, "application/problem+json",
        "unexpected Content-Type for problem response: {normalized}"
    );
}

fn assert_problem_status(response_json: &Value, status: u16) {
    let Some(value) = response_json.get("status").and_then(|value| value.as_u64()) else {
        panic!("problem response missing status field");
    };
    assert_eq!(
        value, status as u64,
        "problem status mismatch: expected {status}, got {value}"
    );
}

fn assert_problem_code(response_json: &Value) {
    let code = response_json
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if code.trim().is_empty() {
        panic!("problem response missing stable error code");
    }
}

const NOT_FOUND_SCHEMA_REF: &str =
    "@@root#/components/responses/NotFound/content/application~1problem+json/schema";
const PROBLEM_DETAILS_SCHEMA_REF: &str = "@@root#/components/schemas/ProblemDetails";
