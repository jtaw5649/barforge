use std::path::Path;

pub fn load_openapi(
    path: impl AsRef<Path>,
) -> Result<serde_yml::Value, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let spec = serde_yml::from_str(&content)?;
    Ok(spec)
}

pub fn load_openapi_spec(
    path: impl AsRef<Path>,
) -> Result<oas3::OpenApiV3Spec, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let spec = oas3::from_yaml(content)?;
    Ok(spec)
}

pub fn load_openapi_json(
    path: impl AsRef<Path>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let yaml_value: serde_yml::Value = serde_yml::from_str(&content)?;
    let json_value = serde_json::to_value(yaml_value)?;
    Ok(json_value)
}
