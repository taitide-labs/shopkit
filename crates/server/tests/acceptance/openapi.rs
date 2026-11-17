use std::path::Path;

use serde_json::{Map, Value};
use shopkit_infra::routing::RunEnv;
use shopkit_server::app::build_openapi;

#[test]
fn registered_openapi_spec_is_valid_json() {
    let spec = build_openapi(RunEnv::Development);
    let json = serde_json::to_value(&spec).expect("serialize openapi");
    let title = json
        .get("info")
        .and_then(|info| info.get("title"))
        .and_then(Value::as_str);
    assert_eq!(title, Some("shopkit"));
    assert!(
        json.get("paths")
            .and_then(Value::as_object)
            .is_some_and(|paths: &Map<String, Value>| paths.len() == 8)
    );
}

#[tokio::test]
#[ignore = "openapi-export"]
async fn export_openapi_spec() {
    let spec = build_openapi(RunEnv::Production);
    let out_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/openapi.json");
    if let Some(parent) = out_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .expect("create contracts dir");
    }
    let formatted = serde_json::to_string_pretty(&spec).expect("pretty openapi json");
    tokio::fs::write(&out_path, format!("{formatted}\n"))
        .await
        .expect("write openapi.json");
}
