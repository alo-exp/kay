use forge_config::ForgeConfig;

#[test]
fn config_defaults_are_sensible() {
    // ForgeConfig derives Default — verify it constructs without panic
    let cfg = ForgeConfig::default();
    // services_url has a dummy default; verify it is set
    assert!(!cfg.services_url.is_empty(), "services_url should have a default");
    assert!(cfg.max_search_lines > 0, "max_search_lines should have a sensible default");
}

#[test]
fn config_serialize_deserialize() {
    let cfg = ForgeConfig::default();
    let json = serde_json::to_string(&cfg).expect("config must serialize");
    let roundtrip: ForgeConfig = serde_json::from_str(&json).expect("config must deserialize");
    assert_eq!(cfg.services_url, roundtrip.services_url);
}
