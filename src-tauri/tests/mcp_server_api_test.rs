// MCP服务器API集成测试
//
// 测试完整的MCP服务器管理API工作流程
// 验证所有CRUD操作和业务逻辑

use reqwest;
use serde_json::{json, Value};

#[tokio::test]
async fn test_mcp_server_complete_workflow() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 1. 测试健康检查
    let health_response = client
        .get(&format!("{}/health", base_url.replace("/api/v1", "")))
        .send()
        .await
        .expect("健康检查请求失败");

    assert_eq!(health_response.status(), 200);

    // 2. 获取初始服务器列表（应为空）
    let list_response = client
        .get(&format!("{}/mcp-servers", base_url))
        .send()
        .await
        .expect("获取服务器列表请求失败");

    assert_eq!(list_response.status(), 200);
    let list_data: Value = list_response.json().await.expect("解析响应失败");
    assert!(list_data["success"].as_bool().unwrap());
    let servers = list_data["data"]["data"].as_array().unwrap();
    assert_eq!(servers.len(), 0);

    // 3. 创建新的MCP服务器 - stdio类型
    let stdio_create_request = json!({
        "name": "集成测试STDIO服务器",
        "description": "这是一个用于集成测试的STDIO类型MCP服务器",
        "command": "node",
        "args": ["server.js"],
        "env": {"NODE_ENV": "test", "DEBUG": "true"},
        "cwd": "/workspace",
        "connection_type": "stdio",
        "timeout": 30000,
        "enabled": 1
    });

    let stdio_create_response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&stdio_create_request)
        .send()
        .await
        .expect("创建STDIO服务器请求失败");

    assert_eq!(stdio_create_response.status(), 200);
    let stdio_create_data: Value = stdio_create_response.json().await.expect("解析创建响应失败");
    assert!(stdio_create_data["success"].as_bool().unwrap());

    let stdio_server_id = stdio_create_data["data"]["id"].as_i64().unwrap();
    assert!(stdio_server_id > 0);

    // 4. 创建第二个MCP服务器 - SSE类型
    let sse_create_request = json!({
        "name": "集成测试SSE服务器",
        "description": "这是一个用于集成测试的SSE类型MCP服务器",
        "url": "http://localhost:3001/mcp",
        "connection_type": "sse",
        "timeout": 60000,
        "enabled": 1,
        "headers": {"Authorization": "Bearer test-token", "Content-Type": "application/json"}
    });

    let sse_create_response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&sse_create_request)
        .send()
        .await
        .expect("创建SSE服务器请求失败");

    assert_eq!(sse_create_response.status(), 200);
    let sse_create_data: Value = sse_create_response.json().await.expect("解析创建响应失败");
    assert!(sse_create_data["success"].as_bool().unwrap());

    let sse_server_id = sse_create_data["data"]["id"].as_i64().unwrap();
    assert!(sse_server_id > 0);

    // 5. 获取特定STDIO服务器
    let get_stdio_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, stdio_server_id))
        .send()
        .await
        .expect("获取STDIO服务器详情请求失败");

    assert_eq!(get_stdio_response.status(), 200);
    let get_stdio_data: Value = get_stdio_response.json().await.expect("解析获取响应失败");
    assert!(get_stdio_data["success"].as_bool().unwrap());
    assert_eq!(get_stdio_data["data"]["name"].as_str().unwrap(), "集成测试STDIO服务器");
    assert_eq!(get_stdio_data["data"]["connection_type"].as_str().unwrap(), "stdio");
    assert_eq!(get_stdio_data["data"]["enabled"].as_i64().unwrap(), 1);

    // 6. 获取特定SSE服务器
    let get_sse_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, sse_server_id))
        .send()
        .await
        .expect("获取SSE服务器详情请求失败");

    assert_eq!(get_sse_response.status(), 200);
    let get_sse_data: Value = get_sse_response.json().await.expect("解析获取响应失败");
    assert!(get_sse_data["success"].as_bool().unwrap());
    assert_eq!(get_sse_data["data"]["name"].as_str().unwrap(), "集成测试SSE服务器");
    assert_eq!(get_sse_data["data"]["connection_type"].as_str().unwrap(), "sse");

    // 7. 更新STDIO服务器
    let stdio_update_request = json!({
        "name": "集成测试STDIO服务器-已更新",
        "description": "这是一个更新后的描述",
        "timeout": 45000,
        "enabled": 0
    });

    let stdio_update_response = client
        .put(&format!("{}/mcp-servers/{}", base_url, stdio_server_id))
        .json(&stdio_update_request)
        .send()
        .await
        .expect("更新STDIO服务器请求失败");

    assert_eq!(stdio_update_response.status(), 200);
    let stdio_update_data: Value = stdio_update_response.json().await.expect("解析更新响应失败");
    assert!(stdio_update_data["success"].as_bool().unwrap());
    assert_eq!(stdio_update_data["data"]["name"].as_str().unwrap(), "集成测试STDIO服务器-已更新");
    assert_eq!(stdio_update_data["data"]["enabled"].as_i64().unwrap(), 0);

    // 8. 测试搜索功能
    let search_response = client
        .get(&format!("{}/mcp-servers?search=集成测试&limit=10", base_url))
        .send()
        .await
        .expect("搜索服务器请求失败");

    assert_eq!(search_response.status(), 200);
    let search_data: Value = search_response.json().await.expect("解析搜索响应失败");
    assert!(search_data["success"].as_bool().unwrap());
    let search_results = search_data["data"]["data"].as_array().unwrap();
    assert_eq!(search_results.len(), 2); // 应该找到两个服务器

    // 9. 测试配置验证
    let test_stdio_response = client
        .get(&format!("{}/mcp-servers/{}/test", base_url, stdio_server_id))
        .send()
        .await
        .expect("测试STDIO服务器配置请求失败");

    assert_eq!(test_stdio_response.status(), 200);
    let test_stdio_data: Value = test_stdio_response.json().await.expect("解析测试响应失败");
    assert!(test_stdio_data["success"].as_bool().unwrap());

    // 10. 测试统计信息
    let stats_response = client
        .get(&format!("{}/mcp-servers/stats", base_url))
        .send()
        .await
        .expect("获取统计信息请求失败");

    assert_eq!(stats_response.status(), 200);
    let stats_data: Value = stats_response.json().await.expect("解析统计信息响应失败");
    assert!(stats_data["success"].as_bool().unwrap());
    assert_eq!(stats_data["data"]["total"].as_i64().unwrap(), 2);
    assert_eq!(stats_data["data"]["stdio_type"].as_i64().unwrap(), 1);
    assert_eq!(stats_data["data"]["sse_type"].as_i64().unwrap(), 1);
    assert_eq!(stats_data["data"]["active_count"].as_i64().unwrap(), 1); // 只有SSE服务器是启用的

    // 11. 测试分页功能
    let pagination_response = client
        .get(&format!("{}/mcp-servers?page=1&limit=1", base_url))
        .send()
        .await
        .expect("分页测试请求失败");

    assert_eq!(pagination_response.status(), 200);
    let pagination_data: Value = pagination_response.json().await.expect("解析分页响应失败");
    assert!(pagination_data["success"].as_bool().unwrap());
    let paginated_servers = pagination_data["data"]["data"].as_array().unwrap();
    assert_eq!(paginated_servers.len(), 1); // 每页只有1条记录
    assert_eq!(pagination_data["data"]["total"].as_i64().unwrap(), 2);

    // 12. 删除STDIO服务器
    let delete_stdio_response = client
        .delete(&format!("{}/mcp-servers/{}", base_url, stdio_server_id))
        .send()
        .await
        .expect("删除STDIO服务器请求失败");

    assert_eq!(delete_stdio_response.status(), 200);
    let delete_stdio_data: Value = delete_stdio_response.json().await.expect("解析删除响应失败");
    assert!(delete_stdio_data["success"].as_bool().unwrap());

    // 13. 删除SSE服务器
    let delete_sse_response = client
        .delete(&format!("{}/mcp-servers/{}", base_url, sse_server_id))
        .send()
        .await
        .expect("删除SSE服务器请求失败");

    assert_eq!(delete_sse_response.status(), 200);
    let delete_sse_data: Value = delete_sse_response.json().await.expect("解析删除响应失败");
    assert!(delete_sse_data["success"].as_bool().unwrap());

    // 14. 验证服务器已被删除
    let verify_stdio_delete_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, stdio_server_id))
        .send()
        .await
        .expect("验证STDIO服务器删除请求失败");

    assert_eq!(verify_stdio_delete_response.status(), 404);

    let verify_sse_delete_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, sse_server_id))
        .send()
        .await
        .expect("验证SSE服务器删除请求失败");

    assert_eq!(verify_sse_delete_response.status(), 404);

    // 15. 验证统计信息已重置
    let final_stats_response = client
        .get(&format!("{}/mcp-servers/stats", base_url))
        .send()
        .await
        .expect("获取最终统计信息请求失败");

    assert_eq!(final_stats_response.status(), 200);
    let final_stats_data: Value = final_stats_response.json().await.expect("解析最终统计信息响应失败");
    assert_eq!(final_stats_data["data"]["total"].as_i64().unwrap(), 0);
    assert_eq!(final_stats_data["data"]["stdio_type"].as_i64().unwrap(), 0);
    assert_eq!(final_stats_data["data"]["sse_type"].as_i64().unwrap(), 0);
    assert_eq!(final_stats_data["data"]["active_count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_mcp_server_stdio_configuration() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试完整的STDIO配置
    let stdio_request = json!({
        "name": "STDIO配置测试",
        "description": "测试STDIO类型服务器的完整配置",
        "command": "python",
        "args": ["-m", "mcp.server", "--port", "8080"],
        "env": {
            "PYTHONPATH": "/workspace/src",
            "LOG_LEVEL": "DEBUG",
            "API_KEY": "test-api-key"
        },
        "cwd": "/workspace",
        "connection_type": "stdio",
        "timeout": 60000,
        "enabled": 1
    });

    let create_response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&stdio_request)
        .send()
        .await
        .expect("创建STDIO配置测试请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建响应失败");
    let server_id = create_data["data"]["id"].as_i64().unwrap();

    // 验证配置保存正确
    let get_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, server_id))
        .send()
        .await
        .expect("获取STDIO配置请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取响应失败");
    assert_eq!(get_data["data"]["command"].as_str().unwrap(), "python");
    assert_eq!(get_data["data"]["connection_type"].as_str().unwrap(), "stdio");
    assert!(get_data["data"]["args"].as_array().unwrap().len() == 3);
    assert!(get_data["data"]["env"].as_object().unwrap().contains_key("PYTHONPATH"));

    // 清理
    let delete_response = client
        .delete(&format!("{}/mcp-servers/{}", base_url, server_id))
        .send()
        .await
        .expect("清理STDIO配置测试请求失败");

    assert_eq!(delete_response.status(), 200);
}

#[tokio::test]
async fn test_mcp_server_sse_configuration() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试完整的SSE配置
    let sse_request = json!({
        "name": "SSE配置测试",
        "description": "测试SSE类型服务器的完整配置",
        "url": "https://api.example.com/mcp",
        "connection_type": "sse",
        "timeout": 120000,
        "enabled": 1,
        "headers": {
            "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
            "Content-Type": "application/json",
            "X-API-Version": "v1"
        }
    });

    let create_response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&sse_request)
        .send()
        .await
        .expect("创建SSE配置测试请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建响应失败");
    let server_id = create_data["data"]["id"].as_i64().unwrap();

    // 验证配置保存正确
    let get_response = client
        .get(&format!("{}/mcp-servers/{}", base_url, server_id))
        .send()
        .await
        .expect("获取SSE配置请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取响应失败");
    assert_eq!(get_data["data"]["url"].as_str().unwrap(), "https://api.example.com/mcp");
    assert_eq!(get_data["data"]["connection_type"].as_str().unwrap(), "sse");
    assert!(get_data["data"]["headers"].as_object().unwrap().contains_key("Authorization"));

    // 清理
    let delete_response = client
        .delete(&format!("{}/mcp-servers/{}", base_url, server_id))
        .send()
        .await
        .expect("清理SSE配置测试请求失败");

    assert_eq!(delete_response.status(), 200);
}

#[tokio::test]
async fn test_mcp_server_validation_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试空名称
    let empty_name_request = json!({
        "name": "",
        "connection_type": "stdio",
        "command": "node"
    });

    let response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&empty_name_request)
        .send()
        .await
        .expect("空名称验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("服务器名称不能为空"));

    // 测试无效的连接类型
    let invalid_type_request = json!({
        "name": "测试服务器",
        "connection_type": "invalid_type"
    });

    let response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&invalid_type_request)
        .send()
        .await
        .expect("无效连接类型验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析连接类型验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试STDIO类型缺少command
    let stdio_no_command_request = json!({
        "name": "STDIO测试服务器",
        "connection_type": "stdio",
        "args": ["--help"]
    });

    let response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&stdio_no_command_request)
        .send()
        .await
        .expect("STDIO缺少command验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析STDIO缺少command验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试SSE类型缺少URL
    let sse_no_url_request = json!({
        "name": "SSE测试服务器",
        "connection_type": "sse",
        "timeout": 60000
    });

    let response = client
        .post(&format!("{}/mcp-servers", base_url))
        .json(&sse_no_url_request)
        .send()
        .await
        .expect("SSE缺少URL验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析SSE缺少URL验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_mcp_server_not_found_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试获取不存在的服务器
    let response = client
        .get(&format!("{}/mcp-servers/99999", base_url))
        .send()
        .await
        .expect("不存在服务器请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("MCP服务器不存在"));

    // 测试更新不存在的服务器
    let update_request = json!({
        "name": "更新的名称"
    });

    let response = client
        .put(&format!("{}/mcp-servers/99999", base_url))
        .json(&update_request)
        .send()
        .await
        .expect("更新不存在服务器请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析更新不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试删除不存在的服务器
    let response = client
        .delete(&format!("{}/mcp-servers/99999", base_url))
        .send()
        .await
        .expect("删除不存在服务器请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析删除不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试测试不存在的服务器配置
    let response = client
        .get(&format!("{}/mcp-servers/99999/test", base_url))
        .send()
        .await
        .expect("测试不存在服务器配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析测试不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}