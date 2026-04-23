use forge_domain::{Model, ModelId};
use serde_json;

#[test]
fn model_round_trips_serde() {
    // Model derives Serialize + Deserialize — test a round-trip
    let model = Model {
        id: ModelId::new("gpt-4"),
        name: Some("GPT-4".to_string()),
        description: None,
        context_length: Some(8192),
        tools_supported: Some(true),
        supports_parallel_tool_calls: Some(true),
        supports_reasoning: Some(false),
        input_modalities: vec![forge_domain::InputModality::Text],
    };

    let json = serde_json::to_string(&model).expect("model must serialize");
    let roundtrip: Model = serde_json::from_str(&json).expect("model must deserialize");
    assert_eq!(model.id, roundtrip.id);
    assert_eq!(model.name, roundtrip.name);
}

#[test]
fn model_id_from_str() {
    let id = ModelId::from("claude-3");
    assert_eq!(id.as_str(), "claude-3");
}
