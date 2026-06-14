//! State contract tests for the Tauri plugin.

use tauri_plugin_llama_crab::{LoadedModelInfo, PluginState};

#[test]
fn plugin_state_tracks_loaded_model_metadata() {
    let state = PluginState::default();
    let info = LoadedModelInfo::new(
        "local".into(),
        "/models/tiny.gguf".into(),
        None,
        Some("balanced".into()),
        None,
        None,
    );

    state.insert_model_for_test(info.clone());

    assert_eq!(state.loaded_model_ids(), vec!["local"]);
    assert_eq!(state.model_info("local").unwrap().path, "/models/tiny.gguf");
    assert_eq!(state.remove_model_for_test("local").unwrap().id, "local");
    assert!(state.loaded_model_ids().is_empty());
}
