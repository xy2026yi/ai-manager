// API集成测试
//
// 测试所有API端点的完整功能，包括正常流程和错误处理

use reqwest;
use serde_json::json;
use tokio;

// API服务器基础URL
const API_BASE: &str = "http://127.0.0.1:8080/api/v1";

/// 测试用的客户端
struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: API_BASE.to_string(),
        }
    }

    async fn get(&self, endpoint: &str) -> reqwest::Response {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.client.get(&url).send().await.unwrap()
    }

    async fn post(&self, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.client.post(&url).json(&body).send().await.unwrap()
    }

    async fn put(&self, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.client.put(&url).json(&body).send().await.unwrap()
    }

    async fn delete(&self, endpoint: &str) -> reqwest::Response {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.client.delete(&url).send().await.unwrap()
    }
}

#[tokio::test]
async fn test_health_check() {
    let response = reqwest::get("http://127.0.0.1:8080/health").await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_api_info() {
    let response = reqwest::get("http://127.0.0.1:8080/api/v1/info").await.unwrap();
    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["name"], "AI Manager API");
    assert_eq!(body["version"], "1.0.0");
}

#[tokio::test]
async fn test_agent_guide_crud() {
    let client = ApiClient::new();

    // 创建Agent指导文件
    let create_data = json!({
        "name": "集成测试指导文件",
        "type": "and",
        "text": "这是一个集成测试用的Agent指导文件，包含完整的功能验证。"
    });

    let response = client.post("agent-guides", create_data).await;
    assert_eq!(response.status(), 200);

    let create_response: serde_json::Value = response.json().await.unwrap();
    assert!(create_response["success"].as_bool().unwrap());
    let guide_id = create_response["data"]["id"].as_i64().unwrap();

    // 获取指导文件详情
    let response = client.get(&format!("agent-guides/{}", guide_id)).await;
    assert_eq!(response.status(), 200);

    let get_response: serde_json::Value = response.json().await.unwrap();
    assert!(get_response["success"].as_bool().unwrap());
    assert_eq!(get_response["data"]["name"], "集成测试指导文件");

    // 更新指导文件
    let update_data = json!({
        "name": "更新后的指导文件名称",
        "text": "这是更新后的指导文件内容"
    });

    let response = client.put(&format!("agent-guides/{}", guide_id), update_data).await;
    assert_eq!(response.status(), 200);

    let update_response: serde_json::Value = response.json().await.unwrap();
    assert!(update_response["success"].as_bool().unwrap());
    assert_eq!(update_response["data"]["name"], "更新后的指导文件名称");

    // 获取指导文件列表
    let response = client.get("agent-guides").await;
    assert_eq!(response.status(), 200);

    let list_response: serde_json::Value = response.json().await.unwrap();
    assert!(list_response["success"].as_bool().unwrap());
    assert!(list_response["data"]["data"].as_array().unwrap().len() > 0);

    // 验证指导文件内容
    let response = client.get(&format!("agent-guides/{}/validate", guide_id)).await;
    assert_eq!(response.status(), 200);

    let validate_response: serde_json::Value = response.json().await.unwrap();
    assert!(validate_response["success"].as_bool().unwrap());
    assert!(validate_response["data"].as_bool().unwrap());

    // 删除指导文件
    let response = client.delete(&format!("agent-guides/{}", guide_id)).await;
    assert_eq!(response.status(), 200);

    let delete_response: serde_json::Value = response.json().await.unwrap();
    assert!(delete_response["success"].as_bool().unwrap());

    // 验证删除成功
    let response = client.get(&format!("agent-guides/{}", guide_id)).await;
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_mcp_server_crud() {
    let client = ApiClient::new();

    // 创建MCP服务器
    let create_data = json!({
        "name": "集成测试MCP服务器",
        "type": "stdio",
        "timeout": 30000,
        "command": "node",
        "args": ["test-server.js", "--port", "3000"]
    });

    let response = client.post("mcp-servers", create_data).await;
    assert_eq!(response.status(), 200);

    let create_response: serde_json::Value = response.json().await.unwrap();
    assert!(create_response["success"].as_bool().unwrap());
    let server_id = create_response["data"]["id"].as_i64().unwrap();

    // 获取服务器详情
    let response = client.get(&format!("mcp-servers/{}", server_id)).await;
    assert_eq!(response.status(), 200);

    let get_response: serde_json::Value = response.json().await.unwrap();
    assert!(get_response["success"].as_bool().unwrap());
    assert_eq!(get_response["data"]["name"], "集成测试MCP服务器");

    // 测试服务器配置
    let response = client.get(&format!("mcp-servers/{}/test", server_id)).await;
    assert_eq!(response.status(), 200);

    let test_response: serde_json::Value = response.json().await.unwrap();
    assert!(test_response["success"].as_bool().unwrap());

    // 获取服务器列表
    let response = client.get("mcp-servers").await;
    assert_eq!(response.status(), 200);

    let list_response: serde_json::Value = response.json().await.unwrap();
    assert!(list_response["success"].as_bool().unwrap());
    assert!(list_response["data"]["data"].as_array().unwrap().len() > 0);

    // 删除服务器
    let response = client.delete(&format!("mcp-servers/{}", server_id)).await;
    assert_eq!(response.status(), 200);

    let delete_response: serde_json::Value = response.json().await.unwrap();
    assert!(delete_response["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_common_config_crud() {
    let client = ApiClient::new();

    // 创建通用配置
    let create_data = json!({
        "key": "integration_test_key",
        "value": "${HOME}/test/path",
        "description": "集成测试配置项",
        "category": "test",
        "is_active": 1
    });

    let response = client.post("common-configs", create_data).await;
    assert_eq!(response.status(), 200);

    let create_response: serde_json::Value = response.json().await.unwrap();
    assert!(create_response["success"].as_bool().unwrap());
    let config_id = create_response["data"]["id"].as_i64().unwrap();

    // 获取配置详情
    let response = client.get(&format!("common-configs/{}", config_id)).await;
    assert_eq!(response.status(), 200);

    let get_response: serde_json::Value = response.json().await.unwrap();
    assert!(get_response["success"].as_bool().unwrap());
    assert_eq!(get_response["data"]["key"], "integration_test_key");

    // 根据key获取配置
    let response = client.get("common-configs/key/integration_test_key").await;
    assert_eq!(response.status(), 200);

    let key_response: serde_json::Value = response.json().await.unwrap();
    assert!(key_response["success"].as_bool().unwrap());
    assert_eq!(key_response["data"]["key"], "integration_test_key");

    // 批量更新配置
    let batch_data = json!({
        "configs": [
            {"key": "integration_test_key", "value": "${HOME}/updated/path"}
        ]
    });

    let response = client.post("common-configs/batch", batch_data).await;
    assert_eq!(response.status(), 200);

    let batch_response: serde_json::Value = response.json().await.unwrap();
    assert!(batch_response["success"].as_bool().unwrap());

    // 验证配置值
    let response = client.get(&format!("common-configs/{}/validate", config_id)).await;
    assert_eq!(response.status(), 200);

    let validate_response: serde_json::Value = response.json().await.unwrap();
    assert!(validate_response["success"].as_bool().unwrap());
    assert!(validate_response["data"].as_bool().unwrap());

    // 删除配置
    let response = client.delete(&format!("common-configs/{}", config_id)).await;
    assert_eq!(response.status(), 200);

    let delete_response: serde_json::Value = response.json().await.unwrap();
    assert!(delete_response["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_stats_endpoints() {
    let client = ApiClient::new();

    // 测试Agent指导文件统计
    let response = client.get("agent-guides/stats").await;
    assert_eq!(response.status(), 200);

    let stats_response: serde_json::Value = response.json().await.unwrap();
    assert!(stats_response["success"].as_bool().unwrap());
    assert!(stats_response["data"]["total"].is_number());

    // 测试MCP服务器统计
    let response = client.get("mcp-servers/stats").await;
    assert_eq!(response.status(), 200);

    let stats_response: serde_json::Value = response.json().await.unwrap();
    assert!(stats_response["success"].as_bool().unwrap());
    assert!(stats_response["data"]["total"].is_number());

    // 测试通用配置统计
    let response = client.get("common-configs/stats").await;
    assert_eq!(response.status(), 200);

    let stats_response: serde_json::Value = response.json().await.unwrap();
    assert!(stats_response["success"].as_bool().unwrap());
    assert!(stats_response["data"]["total"].is_number());
}

#[tokio::test]
async fn test_error_handling() {
    let client = ApiClient::new();

    // 测试不存在的资源
    let response = client.get("agent-guides/99999").await;
    assert_eq!(response.status(), 404);

    let response = client.get("mcp-servers/99999").await;
    assert_eq!(response.status(), 404);

    let response = client.get("common-configs/99999").await;
    assert_eq!(response.status(), 404);

    // 测试无效的请求体
    let invalid_data = json!({
        "name": "",  // 空名称应该导致验证失败
        "type": "invalid_type",
        "text": ""
    });

    let response = client.post("agent-guides", invalid_data).await;
    assert_eq!(response.status(), 400);

    // 测试无效的ID
    let response = client.get("agent-guides/0").await;
    assert_eq!(response.status(), 400);

    let response = client.get("agent-guides/-1").await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_search_and_pagination() {
    let client = ApiClient::new();

    // 测试搜索功能
    let response = client.get("agent-guides?search=test").await;
    assert_eq!(response.status(), 200);

    let search_response: serde_json::Value = response.json().await.unwrap();
    assert!(search_response["success"].as_bool().unwrap());

    let response = client.get("mcp-servers?search=test").await;
    assert_eq!(response.status(), 200);

    let search_response: serde_json::Value = response.json().await.unwrap();
    assert!(search_response["success"].as_bool().unwrap());

    // 测试分页功能
    let response = client.get("agent-guides?page=1&limit=5").await;
    assert_eq!(response.status(), 200);

    let page_response: serde_json::Value = response.json().await.unwrap();
    assert!(page_response["success"].as_bool().unwrap());
    assert_eq!(page_response["data"]["pagination"]["page"], 1);
    assert_eq!(page_response["data"]["pagination"]["limit"], 5);

    // 测试筛选功能
    let response = client.get("agent-guides?guide_type=only").await;
    assert_eq!(response.status(), 200);

    let filter_response: serde_json::Value = response.json().await.unwrap();
    assert!(filter_response["success"].as_bool().unwrap());

    let response = client.get("mcp-servers?server_type=stdio").await;
    assert_eq!(response.status(), 200);

    let filter_response: serde_json::Value = response.json().await.unwrap();
    assert!(filter_response["success"].as_bool().unwrap());

    let response = client.get("common-configs?category=test").await;
    assert_eq!(response.status(), 200);

    let filter_response: serde_json::Value = response.json().await.unwrap();
    assert!(filter_response["success"].as_bool().unwrap());
}
