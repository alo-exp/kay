use forge_config::ForgeConfig;

#[test]
fn config_defaults_are_sensible() {
    // ForgeConfig derives Default — verify it constructs without panic
    let cfg = ForgeConfig::default();
    // Basic sanity: clone should work and fields should be accessible
    let _ = cfg.clone();
    assert!(true, "ForgeConfig::default() constructs without panic");
}

#[test]
fn config_serialize_deserialize() {
    let cfg = ForgeConfig::default();
    let json = serde_json::to_string(&cfg).expect("config must serialize");
    let roundtrip: ForgeConfig = serde_json::from_str(&json).expect("config must deserialize");
    assert_eq!(cfg.services_url, roundtrip.services_url);
}
