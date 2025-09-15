use rjq::{App, AppBuilder, AppConfig};
use serde_json::json;

#[test]
fn test_app_lifecycle() {
    // アプリケーションの基本的なライフサイクルテスト
    let json_data = json!({"name": "integration_test", "value": 42});
    let app = App::new(json_data);

    assert_eq!(app.input(), "");
    assert!(!app.should_exit());
    assert_eq!(app.prompt(), "query > ");
}

#[test]
fn test_custom_config_integration() {
    let json_data = json!({"test": "data"});
    let custom_config = AppConfig::with_prompt("custom > ");
    let app = App::with_config(json_data, custom_config);

    assert_eq!(app.prompt(), "custom > ");
    assert_eq!(app.input(), "");
}

#[test]
fn test_query_execution_flow() {
    let json_data = json!({"items": [1, 2, 3], "name": "test"});
    let app = App::new(json_data);

    // 基本的なクエリ実行のテスト
    // 注意: execute_current_query()は空のクエリでエラーになるため、
    // 実際のクエリ処理は他のテストで行う
    assert!(app.data().get().is_object());
}

#[test]
fn test_error_handling_integration() {
    let json_data = json!({"test": "value"});
    let app = App::new(json_data);

    // エラー状態のテスト
    assert!(app.last_error().is_none());
}

#[test]
fn test_enhanced_app_with_cache() {
    // Phase 3の新機能: キャッシュ付きアプリケーションのテスト
    let json_data = json!({"name": "cache_test", "items": [1, 2, 3]});
    let mut app = AppBuilder::new(json_data).with_cache().build();

    // 基本的な動作確認
    assert_eq!(app.input(), "");
    assert!(!app.should_exit());

    // 文字入力のテスト
    app.push_char('.');
    assert_eq!(app.input(), ".");
}

#[test]
fn test_app_builder_pattern() {
    // Phase 3の新機能: AppBuilderパターンのテスト
    let json_data = json!({"builder": "test"});
    let custom_config = AppConfig::with_prompt("builder > ");

    let app = AppBuilder::new(json_data)
        .with_config(custom_config)
        .build();

    assert_eq!(app.prompt(), "builder > ");
    assert_eq!(app.input(), "");
    assert!(!app.should_exit());
}
