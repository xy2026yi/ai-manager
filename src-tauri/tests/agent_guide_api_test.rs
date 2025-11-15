// Agent指导文件API集成测试
//
// 测试完整的Agent指导文件管理API工作流程
// 验证所有CRUD操作和业务逻辑

use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_agent_guide_complete_workflow() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 1. 测试健康检查
    let health_response = client
        .get(&format!("{}/health", base_url.replace("/api/v1", "")))
        .send()
        .await
        .expect("健康检查请求失败");

    assert_eq!(health_response.status(), 200);

    // 2. 获取初始指导文件列表（应为空）
    let list_response = client
        .get(&format!("{}/agent-guides", base_url))
        .send()
        .await
        .expect("获取指导文件列表请求失败");

    assert_eq!(list_response.status(), 200);
    let list_data: Value = list_response.json().await.expect("解析响应失败");
    assert!(list_data["success"].as_bool().unwrap());
    let guides = list_data["data"]["data"].as_array().unwrap();
    assert_eq!(guides.len(), 0);

    // 3. 创建新的Agent指导文件
    let create_request = json!({
        "name": "集成测试指导文件",
        "description": "这是一个用于集成测试的Agent指导文件",
        "content": "# 集成测试指导文件\\n\\n这是测试内容，用于验证Agent指导文件的创建、读取、更新和删除功能。\\n\\n## 功能特性\\n- 支持Markdown格式\\n- 支持代码高亮\\n- 支持链接和图片",
        "category": "测试类别",
        "version": "1.0.0",
        "author": "集成测试系统"
    });

    let create_response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&create_request)
        .send()
        .await
        .expect("创建指导文件请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建响应失败");
    assert!(create_data["success"].as_bool().unwrap());

    let guide_id = create_data["data"]["id"].as_i64().unwrap();
    assert!(guide_id > 0);

    // 4. 获取特定指导文件
    let get_response = client
        .get(&format!("{}/agent-guides/{}", base_url, guide_id))
        .send()
        .await
        .expect("获取指导文件详情请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取响应失败");
    assert!(get_data["success"].as_bool().unwrap());
    assert_eq!(
        get_data["data"]["name"].as_str().unwrap(),
        "集成测试指导文件"
    );
    assert_eq!(
        get_data["data"]["description"].as_str().unwrap(),
        "这是一个用于集成测试的Agent指导文件"
    );
    assert!(get_data["data"]["content"].as_str().unwrap().contains("集成测试指导文件"));

    // 5. 更新指导文件
    let update_request = json!({
        "name": "集成测试指导文件-已更新",
        "description": "这是一个更新后的描述",
        "content": "# 更新后的集成测试指导文件\\n\\n这是更新后的测试内容。\\n\\n## 新增功能\\n- 支持更多格式\\n- 优化性能\\n- 修复已知问题",
        "version": "1.1.0"
    });

    let update_response = client
        .put(&format!("{}/agent-guides/{}", base_url, guide_id))
        .json(&update_request)
        .send()
        .await
        .expect("更新指导文件请求失败");

    assert_eq!(update_response.status(), 200);
    let update_data: Value = update_response.json().await.expect("解析更新响应失败");
    assert!(update_data["success"].as_bool().unwrap());
    assert_eq!(
        update_data["data"]["name"].as_str().unwrap(),
        "集成测试指导文件-已更新"
    );
    assert_eq!(update_data["data"]["version"].as_str().unwrap(), "1.1.0");

    // 6. 测试搜索功能
    let search_response = client
        .get(&format!(
            "{}/agent-guides?search=集成测试&limit=10",
            base_url
        ))
        .send()
        .await
        .expect("搜索指导文件请求失败");

    assert_eq!(search_response.status(), 200);
    let search_data: Value = search_response.json().await.expect("解析搜索响应失败");
    assert!(search_data["success"].as_bool().unwrap());
    let search_results = search_data["data"]["data"].as_array().unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(
        search_results[0]["name"].as_str().unwrap(),
        "集成测试指导文件-已更新"
    );

    // 7. 测试验证指导文件
    let validate_response = client
        .get(&format!("{}/agent-guides/{}/validate", base_url, guide_id))
        .send()
        .await
        .expect("验证指导文件请求失败");

    assert_eq!(validate_response.status(), 200);
    let validate_data: Value = validate_response.json().await.expect("解析验证响应失败");
    assert!(validate_data["success"].as_bool().unwrap());
    assert!(validate_data["data"]["valid"].as_bool().unwrap());

    // 8. 测试统计信息
    let stats_response = client
        .get(&format!("{}/agent-guides/stats", base_url))
        .send()
        .await
        .expect("获取统计信息请求失败");

    assert_eq!(stats_response.status(), 200);
    let stats_data: Value = stats_response.json().await.expect("解析统计信息响应失败");
    assert!(stats_data["success"].as_bool().unwrap());
    assert_eq!(stats_data["data"]["total"].as_i64().unwrap(), 1);

    // 9. 创建第二个指导文件用于测试分页
    let second_guide_request = json!({
        "name": "第二个测试指导文件",
        "description": "用于测试分页功能的指导文件",
        "content": "这是第二个测试指导文件的内容",
        "category": "分页测试"
    });

    let second_create_response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&second_guide_request)
        .send()
        .await
        .expect("创建第二个指导文件请求失败");

    assert_eq!(second_create_response.status(), 200);

    // 10. 测试分页功能
    let pagination_response = client
        .get(&format!("{}/agent-guides?page=1&limit=1", base_url))
        .send()
        .await
        .expect("分页测试请求失败");

    assert_eq!(pagination_response.status(), 200);
    let pagination_data: Value = pagination_response.json().await.expect("解析分页响应失败");
    assert!(pagination_data["success"].as_bool().unwrap());
    let paginated_guides = pagination_data["data"]["data"].as_array().unwrap();
    assert_eq!(paginated_guides.len(), 1); // 每页只有1条记录
    assert!(pagination_data["data"]["total"].as_i64().unwrap() >= 2);

    // 11. 删除第一个指导文件
    let delete_response = client
        .delete(&format!("{}/agent-guides/{}", base_url, guide_id))
        .send()
        .await
        .expect("删除指导文件请求失败");

    assert_eq!(delete_response.status(), 200);
    let delete_data: Value = delete_response.json().await.expect("解析删除响应失败");
    assert!(delete_data["success"].as_bool().unwrap());

    // 12. 验证指导文件已被删除
    let verify_delete_response = client
        .get(&format!("{}/agent-guides/{}", base_url, guide_id))
        .send()
        .await
        .expect("验证删除请求失败");

    assert_eq!(verify_delete_response.status(), 404);

    // 13. 清理：删除第二个指导文件
    let second_response_data: Value = second_create_response.json().await.unwrap();
    let second_guide_id = second_response_data["data"]["id"].as_i64().unwrap();
    let cleanup_response = client
        .delete(&format!("{}/agent-guides/{}", base_url, second_guide_id))
        .send()
        .await
        .expect("清理第二个指导文件请求失败");

    assert_eq!(cleanup_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_agent_guide_validation_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试空名称
    let empty_name_request = json!({
        "name": "",
        "description": "测试描述",
        "content": "测试内容"
    });

    let response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&empty_name_request)
        .send()
        .await
        .expect("空名称验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("指导文件名称不能为空"));

    // 测试空内容
    let empty_content_request = json!({
        "name": "测试指导文件",
        "description": "测试描述",
        "content": ""
    });

    let response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&empty_content_request)
        .send()
        .await
        .expect("空内容验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析内容验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("指导文件内容不能为空"));
}

#[tokio::test]
async fn test_agent_guide_not_found_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试获取不存在的指导文件
    let response = client
        .get(&format!("{}/agent-guides/99999", base_url))
        .send()
        .await
        .expect("不存在指导文件请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("Agent指导文件不存在"));

    // 测试更新不存在的指导文件
    let update_request = json!({
        "name": "更新的名称"
    });

    let response = client
        .put(&format!("{}/agent-guides/99999", base_url))
        .json(&update_request)
        .send()
        .await
        .expect("更新不存在指导文件请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析更新不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试删除不存在的指导文件
    let response = client
        .delete(&format!("{}/agent-guides/99999", base_url))
        .send()
        .await
        .expect("删除不存在指导文件请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析删除不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试验证不存在的指导文件
    let response = client
        .get(&format!("{}/agent-guides/99999/validate", base_url))
        .send()
        .await
        .expect("验证不存在指导文件请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析验证不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_agent_guide_content_validation() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 1. 创建包含有效Markdown的指导文件
    let valid_markdown_request = json!({
        "name": "Markdown测试指导文件",
        "description": "测试Markdown格式支持",
        "content": "# 标题\\n\\n这是一个段落。\\n\\n## 子标题\\n\\n- 列表项1\\n- 列表项2\\n\\n```javascript\\nconsole.log('Hello World');\\n```\\n\\n[链接](https://example.com)",
        "category": "格式测试"
    });

    let create_response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&valid_markdown_request)
        .send()
        .await
        .expect("创建Markdown测试指导文件请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建响应失败");
    let guide_id = create_data["data"]["id"].as_i64().unwrap();

    // 2. 验证Markdown内容
    let validate_response = client
        .get(&format!("{}/agent-guides/{}/validate", base_url, guide_id))
        .send()
        .await
        .expect("验证Markdown内容请求失败");

    assert_eq!(validate_response.status(), 200);
    let validate_data: Value = validate_response.json().await.expect("解析验证响应失败");
    assert!(validate_data["success"].as_bool().unwrap());
    assert!(validate_data["data"]["valid"].as_bool().unwrap());

    // 3. 创建包含无效内容的指导文件
    let invalid_content_request = json!({
        "name": "无效内容测试指导文件",
        "description": "测试无效内容处理",
        "content": "",
        "category": "错误测试"
    });

    let invalid_response = client
        .post(&format!("{}/agent-guides", base_url))
        .json(&invalid_content_request)
        .send()
        .await
        .expect("创建无效内容指导文件请求失败");

    assert_eq!(invalid_response.status(), 400);
    let invalid_data: Value = invalid_response.json().await.expect("解析无效内容响应失败");
    assert!(!invalid_data["success"].as_bool().unwrap());

    // 4. 清理
    let delete_response = client
        .delete(&format!("{}/agent-guides/{}", base_url, guide_id))
        .send()
        .await
        .expect("清理Markdown测试指导文件请求失败");

    assert_eq!(delete_response.status(), 200);
}
