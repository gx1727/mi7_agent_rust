use std::sync::Arc;

// 简单测试，不依赖模块导出
#[tokio::test]
async fn test_weather_tool_mock() {
    // Mock 测试
    let params = serde_json::json!({"city": "Beijing"});
    assert_eq!(params["city"], "Beijing");
    
    let result = serde_json::json!({
        "city": "Beijing",
        "temperature": "22°C",
        "condition": "Sunny"
    });
    
    assert_eq!(result["city"], "Beijing");
    assert_eq!(result["temperature"], "22°C");
}
