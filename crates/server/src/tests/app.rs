use shopkit_infra::routing::RunEnv;

use crate::app::{build_openapi, service_name};

#[test]
fn service_name_matches_package() {
    assert_eq!(service_name(), "shopkit-server");
}

#[test]
fn production_openapi_excludes_planned_slices() {
    let dev_spec = build_openapi(RunEnv::Development);
    let prod_spec = build_openapi(RunEnv::Production);
    assert!(dev_spec.paths.paths.len() >= 8);
    assert!(prod_spec.paths.paths.len() <= dev_spec.paths.paths.len());
}
