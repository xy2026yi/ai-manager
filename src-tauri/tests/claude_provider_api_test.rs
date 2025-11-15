// Claude供应商API集成测试
//
// 测试完整的Claude供应商管理API工作流程
// 验证所有CRUD操作和业务逻辑

use reqwest;
use serde_json::{json, Value};

#[tokio::test]
async fn test_claude_provider_complete_workflow() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 1. 测试健康检查
    let health_response = client
        .get(&format!("{}/health", base_url.replace("/api/v1", "")))
        .send()
        .await
        .expect("健康检查请求失败");

    assert_eq!(health_response.status(), 200);

    // 2. 获取初始供应商列表（应为空）
    let list_response = client
        .get(&format!("{}/claude-providers", base_url))
        .send()
        .await
        .expect("获取供应商列表请求失败");

    assert_eq!(list_response.status(), 200);
    let list_data: Value = list_response.json().await.expect("解析响应失败");
    assert!(list_data["success"].as_bool().unwrap());
    let providers = list_data["data"]["data"].as_array().unwrap();
    assert_eq!(providers.len(), 0);

    // 3. 创建新的Claude供应商
    let create_request = json!({
        "name": "集成测试供应商",
        "url": "https://api.anthropic.com",
        "token": "sk-ant-api03-integration-test-key",
        "timeout": 30000,
        "auto_update": 1,
        "type": "public_welfare",
        "opus_model": "claude-3-opus-20240229",
        "sonnet_model": "claude-3-sonnet-20241022",
        "haiku_model": "claude-3-haiku-20240307"
    });

    let create_response = client
        .post(&format!("{}/claude-providers", base_url))
        .json(&create_request)
        .send()
        .await
        .expect("创建供应商请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建响应失败");
    assert!(create_data["success"].as_bool().unwrap());

    let provider_id = create_data["data"]["id"].as_i64().unwrap();
    assert!(provider_id > 0);

    // 4. 获取特定供应商
    let get_response = client
        .get(&format!("{}/claude-providers/{}", base_url, provider_id))
        .send()
        .await
        .expect("获取供应商详情请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取响应失败");
    assert!(get_data["success"].as_bool().unwrap());
    assert_eq!(get_data["data"]["name"].as_str().unwrap(), "集成测试供应商");
    assert_eq!(get_data["data"]["enabled"].as_i64().unwrap(), 1); // 默认启用

    // 5. 更新供应商
    let update_request = json!({
        "name": "集成测试供应商-已更新",
        "timeout": 45000,
        "auto_update": 0
    });

    let update_response = client
        .put(&format!("{}/claude-providers/{}", base_url, provider_id))
        .json(&update_request)
        .send()
        .await
        .expect("更新供应商请求失败");

    assert_eq!(update_response.status(), 200);
    let update_data: Value = update_response.json().await.expect("解析更新响应失败");
    assert!(update_data["success"].as_bool().unwrap());
    assert_eq!(
        update_data["data"]["name"].as_str().unwrap(),
        "集成测试供应商-已更新"
    );
    assert_eq!(update_data["data"]["timeout"].as_i64().unwrap(), 45000);
    assert_eq!(update_data["data"]["auto_update"].as_i64().unwrap(), 0);

    // 6. 测试搜索功能
    let search_response = client
        .get(&format!(
            "{}/claude-providers?search=集成测试&limit=10",
            base_url
        ))
        .send()
        .await
        .expect("搜索供应商请求失败");

    assert_eq!(search_response.status(), 200);
    let search_data: Value = search_response.json().await.expect("解析搜索响应失败");
    assert!(search_data["success"].as_bool().unwrap());
    let search_results = search_data["data"]["data"].as_array().unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(
        search_results[0]["name"].as_str().unwrap(),
        "集成测试供应商-已更新"
    );

    // 7. 测试连接测试
    let test_response = client
        .get(&format!(
            "{}/claude-providers/{}/test",
            base_url, provider_id
        ))
        .send()
        .await
        .expect("连接测试请求失败");

    assert_eq!(test_response.status(), 200);
    let test_data: Value = test_response.json().await.expect("解析连接测试响应失败");
    assert!(test_data["success"].as_bool().unwrap());
    assert!(test_data["data"].as_bool().unwrap());

    // 8. 测试统计信息
    let stats_response = client
        .get(&format!("{}/claude-providers/stats", base_url))
        .send()
        .await
        .expect("获取统计信息请求失败");

    assert_eq!(stats_response.status(), 200);
    let stats_data: Value = stats_response.json().await.expect("解析统计信息响应失败");
    assert!(stats_data["success"].as_bool().unwrap());
    assert_eq!(stats_data["data"]["total"].as_i64().unwrap(), 1);
    assert_eq!(stats_data["data"]["active"].as_i64().unwrap(), 1);
    assert_eq!(stats_data["data"]["inactive"].as_i64().unwrap(), 0);

    // 9. 测试获取当前启用的供应商
    let current_response = client
        .get(&format!("{}/claude-providers/current", base_url))
        .send()
        .await
        .expect("获取当前供应商请求失败");

    assert_eq!(current_response.status(), 200);
    let current_data: Value = current_response.json().await.expect("解析当前供应商响应失败");
    assert!(current_data["success"].as_bool().unwrap());
    assert_eq!(current_data["data"]["id"].as_i64().unwrap(), provider_id);

    // 10. 测试禁用供应商
    let disable_response = client
        .post(&format!(
            "{}/claude-providers/{}/disable",
            base_url, provider_id
        ))
        .send()
        .await
        .expect("禁用供应商请求失败");

    assert_eq!(disable_response.status(), 200);
    let disable_data: Value = disable_response.json().await.expect("解析禁用响应失败");
    assert!(disable_data["success"].as_bool().unwrap());

    // 11. 验证供应商已被禁用
    let get_disabled_response = client
        .get(&format!("{}/claude-providers/{}", base_url, provider_id))
        .send()
        .await
        .expect("获取已禁用供应商请求失败");

    assert_eq!(get_disabled_response.status(), 200);
    let disabled_data: Value =
        get_disabled_response.json().await.expect("解析已禁用供应商响应失败");
    assert_eq!(disabled_data["data"]["enabled"].as_i64().unwrap(), 0);

    // 12. 重新启用供应商
    let enable_response = client
        .post(&format!(
            "{}/claude-providers/{}/enable",
            base_url, provider_id
        ))
        .send()
        .await
        .expect("启用供应商请求失败");

    assert_eq!(enable_response.status(), 200);
    let enable_data: Value = enable_response.json().await.expect("解析启用响应失败");
    assert!(enable_data["success"].as_bool().unwrap());

    // 13. 验证供应商已重新启用
    let get_enabled_response = client
        .get(&format!("{}/claude-providers/{}", base_url, provider_id))
        .send()
        .await
        .expect("获取已启用供应商请求失败");

    assert_eq!(get_enabled_response.status(), 200);
    let enabled_data: Value = get_enabled_response.json().await.expect("解析已启用供应商响应失败");
    assert_eq!(enabled_data["data"]["enabled"].as_i64().unwrap(), 1);

    // 14. 删除供应商
    let delete_response = client
        .delete(&format!("{}/claude-providers/{}", base_url, provider_id))
        .send()
        .await
        .expect("删除供应商请求失败");

    assert_eq!(delete_response.status(), 200);
    let delete_data: Value = delete_response.json().await.expect("解析删除响应失败");
    assert!(delete_data["success"].as_bool().unwrap());

    // 15. 验证供应商已被删除
    let verify_delete_response = client
        .get(&format!("{}/claude-providers/{}", base_url, provider_id))
        .send()
        .await
        .expect("验证删除请求失败");

    assert_eq!(verify_delete_response.status(), 404);

    // 16. 验证统计信息已重置
    let final_stats_response = client
        .get(&format!("{}/claude-providers/stats", base_url))
        .send()
        .await
        .expect("获取最终统计信息请求失败");

    assert_eq!(final_stats_response.status(), 200);
    let final_stats_data: Value =
        final_stats_response.json().await.expect("解析最终统计信息响应失败");
    assert_eq!(final_stats_data["data"]["total"].as_i64().unwrap(), 0);
    assert_eq!(final_stats_data["data"]["active"].as_i64().unwrap(), 0);
    assert_eq!(final_stats_data["data"]["inactive"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_validation_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试空名称
    let empty_name_request = json!({
        "name": "",
        "url": "https://api.anthropic.com",
        "token": "sk-test-key"
    });

    let response = client
        .post(&format!("{}/claude-providers", base_url))
        .json(&empty_name_request)
        .send()
        .await
        .expect("空名称验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("供应商名称不能为空"));

    // 测试无效URL
    let invalid_url_request = json!({
        "name": "测试供应商",
        "url": "invalid-url",
        "token": "sk-test-key"
    });

    let response = client
        .post(&format!("{}/claude-providers", base_url))
        .json(&invalid_url_request)
        .send()
        .await
        .expect("无效URL验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析URL验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"]
        .as_str()
        .unwrap()
        .contains("供应商URL必须以http://或https://开头"));

    // 测试空token
    let empty_token_request = json!({
        "name": "测试供应商",
        "url": "https://api.anthropic.com",
        "token": ""
    });

    let response = client
        .post(&format!("{}/claude-providers", base_url))
        .json(&empty_token_request)
        .send()
        .await
        .expect("空token验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析token验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("供应商Token不能为空"));
}

#[tokio::test]
async fn test_not_found_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试获取不存在的供应商
    let response = client
        .get(&format!("{}/claude-providers/99999", base_url))
        .send()
        .await
        .expect("不存在供应商请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("Claude供应商不存在"));

    // 测试更新不存在的供应商
    let update_request = json!({
        "name": "更新的名称"
    });

    let response = client
        .put(&format!("{}/claude-providers/99999", base_url))
        .json(&update_request)
        .send()
        .await
        .expect("更新不存在供应商请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析更新不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试删除不存在的供应商
    let response = client
        .delete(&format!("{}/claude-providers/99999", base_url))
        .send()
        .await
        .expect("删除不存在供应商请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析删除不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}
