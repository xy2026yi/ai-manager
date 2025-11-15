// 通用配置API集成测试
//
// 测试完整的通用配置管理API工作流程
// 验证所有CRUD操作、批量更新和业务逻辑

use reqwest;
use serde_json::{json, Value};

#[tokio::test]
async fn test_common_config_complete_workflow() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 1. 测试健康检查
    let health_response = client
        .get(&format!("{}/health", base_url.replace("/api/v1", "")))
        .send()
        .await
        .expect("健康检查请求失败");

    assert_eq!(health_response.status(), 200);

    // 2. 获取初始配置列表（应为空）
    let list_response = client
        .get(&format!("{}/common-configs", base_url))
        .send()
        .await
        .expect("获取配置列表请求失败");

    assert_eq!(list_response.status(), 200);
    let list_data: Value = list_response.json().await.expect("解析响应失败");
    assert!(list_data["success"].as_bool().unwrap());
    let configs = list_data["data"]["data"].as_array().unwrap();
    assert_eq!(configs.len(), 0);

    // 3. 创建第一个通用配置
    let first_config_request = json!({
        "key": "app.name",
        "value": "AI Manager",
        "description": "应用程序名称",
        "category": "应用程序",
        "data_type": "string",
        "is_encrypted": 0,
        "enabled": 1
    });

    let first_create_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&first_config_request)
        .send()
        .await
        .expect("创建第一个配置请求失败");

    assert_eq!(first_create_response.status(), 200);
    let first_create_data: Value = first_create_response.json().await.expect("解析创建响应失败");
    assert!(first_create_data["success"].as_bool().unwrap());

    let first_config_id = first_create_data["data"]["id"].as_i64().unwrap();
    assert!(first_config_id > 0);

    // 4. 创建第二个通用配置（加密）
    let second_config_request = json!({
        "key": "database.password",
        "value": "super_secret_password_123",
        "description": "数据库连接密码",
        "category": "数据库",
        "data_type": "string",
        "is_encrypted": 1,
        "enabled": 1
    });

    let second_create_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&second_config_request)
        .send()
        .await
        .expect("创建第二个配置请求失败");

    assert_eq!(second_create_response.status(), 200);
    let second_create_data: Value = second_create_response.json().await.expect("解析创建响应失败");
    assert!(second_create_data["success"].as_bool().unwrap());

    let second_config_id = second_create_data["data"]["id"].as_i64().unwrap();
    assert!(second_config_id > 0);

    // 5. 创建第三个通用配置（数字类型）
    let third_config_request = json!({
        "key": "api.timeout",
        "value": "30000",
        "description": "API请求超时时间（毫秒）",
        "category": "API",
        "data_type": "integer",
        "is_encrypted": 0,
        "enabled": 1
    });

    let third_create_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&third_config_request)
        .send()
        .await
        .expect("创建第三个配置请求失败");

    assert_eq!(third_create_response.status(), 200);
    let third_create_data: Value = third_create_response.json().await.expect("解析创建响应失败");
    assert!(third_create_data["success"].as_bool().unwrap());

    let third_config_id = third_create_data["data"]["id"].as_i64().unwrap();
    assert!(third_config_id > 0);

    // 6. 获取特定配置
    let get_response = client
        .get(&format!("{}/common-configs/{}", base_url, first_config_id))
        .send()
        .await
        .expect("获取配置详情请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取响应失败");
    assert!(get_data["success"].as_bool().unwrap());
    assert_eq!(get_data["data"]["key"].as_str().unwrap(), "app.name");
    assert_eq!(get_data["data"]["value"].as_str().unwrap(), "AI Manager");
    assert_eq!(get_data["data"]["data_type"].as_str().unwrap(), "string");

    // 7. 根据key获取配置
    let get_by_key_response = client
        .get(&format!("{}/common-configs/key/app.name", base_url))
        .send()
        .await
        .expect("根据key获取配置请求失败");

    assert_eq!(get_by_key_response.status(), 200);
    let get_by_key_data: Value = get_by_key_response.json().await.expect("解析根据key获取响应失败");
    assert!(get_by_key_data["success"].as_bool().unwrap());
    assert_eq!(get_by_key_data["data"]["key"].as_str().unwrap(), "app.name");

    // 8. 更新配置
    let update_request = json!({
        "value": "AI Manager Pro",
        "description": "更新后的应用程序名称"
    });

    let update_response = client
        .put(&format!("{}/common-configs/{}", base_url, first_config_id))
        .json(&update_request)
        .send()
        .await
        .expect("更新配置请求失败");

    assert_eq!(update_response.status(), 200);
    let update_data: Value = update_response.json().await.expect("解析更新响应失败");
    assert!(update_data["success"].as_bool().unwrap());
    assert_eq!(
        update_data["data"]["value"].as_str().unwrap(),
        "AI Manager Pro"
    );

    // 9. 测试搜索功能
    let search_response = client
        .get(&format!(
            "{}/common-configs?search=应用程序&limit=10",
            base_url
        ))
        .send()
        .await
        .expect("搜索配置请求失败");

    assert_eq!(search_response.status(), 200);
    let search_data: Value = search_response.json().await.expect("解析搜索响应失败");
    assert!(search_data["success"].as_bool().unwrap());
    let search_results = search_data["data"]["data"].as_array().unwrap();
    assert_eq!(search_results.len(), 1);

    // 10. 测试验证配置值
    let validate_response = client
        .get(&format!(
            "{}/common-configs/{}/validate",
            base_url, first_config_id
        ))
        .send()
        .await
        .expect("验证配置值请求失败");

    assert_eq!(validate_response.status(), 200);
    let validate_data: Value = validate_response.json().await.expect("解析验证响应失败");
    assert!(validate_data["success"].as_bool().unwrap());
    assert!(validate_data["data"]["valid"].as_bool().unwrap());

    // 11. 测试统计信息
    let stats_response = client
        .get(&format!("{}/common-configs/stats", base_url))
        .send()
        .await
        .expect("获取统计信息请求失败");

    assert_eq!(stats_response.status(), 200);
    let stats_data: Value = stats_response.json().await.expect("解析统计信息响应失败");
    assert!(stats_data["success"].as_bool().unwrap());
    assert_eq!(stats_data["data"]["total"].as_i64().unwrap(), 3);
    assert_eq!(stats_data["data"]["active"].as_i64().unwrap(), 3);
    assert_eq!(stats_data["data"]["inactive"].as_i64().unwrap(), 0);

    // 12. 测试批量更新
    let batch_update_request = json!({
        "configs": [
            {
                "id": second_config_id,
                "enabled": 0
            },
            {
                "id": third_config_id,
                "value": "60000",
                "description": "更新后的API超时时间"
            }
        ]
    });

    let batch_update_response = client
        .post(&format!("{}/common-configs/batch", base_url))
        .json(&batch_update_request)
        .send()
        .await
        .expect("批量更新配置请求失败");

    assert_eq!(batch_update_response.status(), 200);
    let batch_update_data: Value =
        batch_update_response.json().await.expect("解析批量更新响应失败");
    assert!(batch_update_data["success"].as_bool().unwrap());
    let updated_configs = batch_update_data["data"]["updated"].as_array().unwrap();
    assert_eq!(updated_configs.len(), 2);

    // 13. 验证批量更新结果
    let verify_second_response = client
        .get(&format!("{}/common-configs/{}", base_url, second_config_id))
        .send()
        .await
        .expect("验证第二个配置更新请求失败");

    assert_eq!(verify_second_response.status(), 200);
    let verify_second_data: Value =
        verify_second_response.json().await.expect("解析验证第二个配置响应失败");
    assert_eq!(verify_second_data["data"]["enabled"].as_i64().unwrap(), 0);

    // 14. 删除配置
    let delete_response = client
        .delete(&format!("{}/common-configs/{}", base_url, first_config_id))
        .send()
        .await
        .expect("删除配置请求失败");

    assert_eq!(delete_response.status(), 200);
    let delete_data: Value = delete_response.json().await.expect("解析删除响应失败");
    assert!(delete_data["success"].as_bool().unwrap());

    // 15. 清理其他配置
    let delete_second_response = client
        .delete(&format!("{}/common-configs/{}", base_url, second_config_id))
        .send()
        .await
        .expect("删除第二个配置请求失败");

    let delete_third_response = client
        .delete(&format!("{}/common-configs/{}", base_url, third_config_id))
        .send()
        .await
        .expect("删除第三个配置请求失败");

    assert_eq!(delete_second_response.status(), 200);
    assert_eq!(delete_third_response.status(), 200);

    // 16. 验证统计信息已重置
    let final_stats_response = client
        .get(&format!("{}/common-configs/stats", base_url))
        .send()
        .await
        .expect("获取最终统计信息请求失败");

    assert_eq!(final_stats_response.status(), 200);
    let final_stats_data: Value =
        final_stats_response.json().await.expect("解析最终统计信息响应失败");
    assert_eq!(final_stats_data["data"]["total"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_common_config_data_types() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试字符串类型
    let string_request = json!({
        "key": "test.string",
        "value": "这是一个测试字符串",
        "description": "字符串类型测试",
        "category": "类型测试",
        "data_type": "string",
        "is_encrypted": 0,
        "enabled": 1
    });

    let string_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&string_request)
        .send()
        .await
        .expect("创建字符串类型配置请求失败");

    assert_eq!(string_response.status(), 200);
    let string_data: Value = string_response.json().await.expect("解析字符串类型响应失败");
    let string_id = string_data["data"]["id"].as_i64().unwrap();

    // 测试整数类型
    let integer_request = json!({
        "key": "test.integer",
        "value": "42",
        "description": "整数类型测试",
        "category": "类型测试",
        "data_type": "integer",
        "is_encrypted": 0,
        "enabled": 1
    });

    let integer_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&integer_request)
        .send()
        .await
        .expect("创建整数类型配置请求失败");

    assert_eq!(integer_response.status(), 200);
    let integer_data: Value = integer_response.json().await.expect("解析整数类型响应失败");
    let integer_id = integer_data["data"]["id"].as_i64().unwrap();

    // 测试布尔类型
    let boolean_request = json!({
        "key": "test.boolean",
        "value": "true",
        "description": "布尔类型测试",
        "category": "类型测试",
        "data_type": "boolean",
        "is_encrypted": 0,
        "enabled": 1
    });

    let boolean_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&boolean_request)
        .send()
        .await
        .expect("创建布尔类型配置请求失败");

    assert_eq!(boolean_response.status(), 200);
    let boolean_data: Value = boolean_response.json().await.expect("解析布尔类型响应失败");
    let boolean_id = boolean_data["data"]["id"].as_i64().unwrap();

    // 测试JSON类型
    let json_request = json!({
        "key": "test.json",
        "value": "{\"key\": \"value\", \"number\": 123, \"array\": [1, 2, 3]}",
        "description": "JSON类型测试",
        "category": "类型测试",
        "data_type": "json",
        "is_encrypted": 0,
        "enabled": 1
    });

    let json_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&json_request)
        .send()
        .await
        .expect("创建JSON类型配置请求失败");

    assert_eq!(json_response.status(), 200);
    let json_data: Value = json_response.json().await.expect("解析JSON类型响应失败");
    let json_id = json_data["data"]["id"].as_i64().unwrap();

    // 验证所有类型都保存正确
    let string_get_response = client
        .get(&format!("{}/common-configs/{}", base_url, string_id))
        .send()
        .await
        .expect("获取字符串类型配置请求失败");

    let string_get_data: Value =
        string_get_response.json().await.expect("解析获取字符串类型响应失败");
    assert_eq!(
        string_get_data["data"]["data_type"].as_str().unwrap(),
        "string"
    );

    let integer_get_response = client
        .get(&format!("{}/common-configs/{}", base_url, integer_id))
        .send()
        .await
        .expect("获取整数类型配置请求失败");

    let integer_get_data: Value =
        integer_get_response.json().await.expect("解析获取整数类型响应失败");
    assert_eq!(
        integer_get_data["data"]["data_type"].as_str().unwrap(),
        "integer"
    );

    let boolean_get_response = client
        .get(&format!("{}/common-configs/{}", base_url, boolean_id))
        .send()
        .await
        .expect("获取布尔类型配置请求失败");

    let boolean_get_data: Value =
        boolean_get_response.json().await.expect("解析获取布尔类型响应失败");
    assert_eq!(
        boolean_get_data["data"]["data_type"].as_str().unwrap(),
        "boolean"
    );

    let json_get_response = client
        .get(&format!("{}/common-configs/{}", base_url, json_id))
        .send()
        .await
        .expect("获取JSON类型配置请求失败");

    let json_get_data: Value = json_get_response.json().await.expect("解析获取JSON类型响应失败");
    assert_eq!(json_get_data["data"]["data_type"].as_str().unwrap(), "json");

    // 清理测试数据
    for id in [string_id, integer_id, boolean_id, json_id] {
        let delete_response = client
            .delete(&format!("{}/common-configs/{}", base_url, id))
            .send()
            .await
            .expect("清理类型测试配置请求失败");
        assert_eq!(delete_response.status(), 200);
    }
}

#[tokio::test]
async fn test_common_config_encryption() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 创建加密配置
    let encrypted_request = json!({
        "key": "secret.api_key",
        "value": "sk-1234567890abcdef",
        "description": "加密的API密钥",
        "category": "安全",
        "data_type": "string",
        "is_encrypted": 1,
        "enabled": 1
    });

    let create_response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&encrypted_request)
        .send()
        .await
        .expect("创建加密配置请求失败");

    assert_eq!(create_response.status(), 200);
    let create_data: Value = create_response.json().await.expect("解析创建加密配置响应失败");
    let config_id = create_data["data"]["id"].as_i64().unwrap();

    // 验证配置已加密（返回的值应该是加密后的）
    let get_response = client
        .get(&format!("{}/common-configs/{}", base_url, config_id))
        .send()
        .await
        .expect("获取加密配置请求失败");

    assert_eq!(get_response.status(), 200);
    let get_data: Value = get_response.json().await.expect("解析获取加密配置响应失败");
    assert!(get_data["success"].as_bool().unwrap());
    assert_eq!(get_data["data"]["is_encrypted"].as_i64().unwrap(), 1);
    // 加密的值不应该与原始值相同
    assert_ne!(
        get_data["data"]["value"].as_str().unwrap(),
        "sk-1234567890abcdef"
    );

    // 清理
    let delete_response = client
        .delete(&format!("{}/common-configs/{}", base_url, config_id))
        .send()
        .await
        .expect("删除加密配置请求失败");

    assert_eq!(delete_response.status(), 200);
}

#[tokio::test]
async fn test_common_config_validation_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试空key
    let empty_key_request = json!({
        "key": "",
        "value": "test_value",
        "description": "测试描述"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&empty_key_request)
        .send()
        .await
        .expect("空key验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("配置键不能为空"));

    // 测试空值
    let empty_value_request = json!({
        "key": "test.key",
        "value": "",
        "description": "测试描述"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&empty_value_request)
        .send()
        .await
        .expect("空值验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析空值验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("配置值不能为空"));

    // 测试无效的数据类型
    let invalid_type_request = json!({
        "key": "test.invalid",
        "value": "test_value",
        "description": "测试无效类型",
        "data_type": "invalid_type"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&invalid_type_request)
        .send()
        .await
        .expect("无效类型验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析无效类型验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试整数类型验证
    let invalid_integer_request = json!({
        "key": "test.integer.invalid",
        "value": "not_an_integer",
        "description": "无效整数测试",
        "data_type": "integer"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&invalid_integer_request)
        .send()
        .await
        .expect("无效整数验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析无效整数验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试布尔类型验证
    let invalid_boolean_request = json!({
        "key": "test.boolean.invalid",
        "value": "not_a_boolean",
        "description": "无效布尔测试",
        "data_type": "boolean"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&invalid_boolean_request)
        .send()
        .await
        .expect("无效布尔验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析无效布尔验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试JSON类型验证
    let invalid_json_request = json!({
        "key": "test.json.invalid",
        "value": "invalid_json_string",
        "description": "无效JSON测试",
        "data_type": "json"
    });

    let response = client
        .post(&format!("{}/common-configs", base_url))
        .json(&invalid_json_request)
        .send()
        .await
        .expect("无效JSON验证请求失败");

    assert_eq!(response.status(), 400);
    let data: Value = response.json().await.expect("解析无效JSON验证错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_common_config_not_found_errors() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 测试获取不存在的配置
    let response = client
        .get(&format!("{}/common-configs/99999", base_url))
        .send()
        .await
        .expect("不存在配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
    assert!(data["error"]["message"].as_str().unwrap().contains("通用配置不存在"));

    // 测试更新不存在的配置
    let update_request = json!({
        "value": "更新的值"
    });

    let response = client
        .put(&format!("{}/common-configs/99999", base_url))
        .json(&update_request)
        .send()
        .await
        .expect("更新不存在配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析更新不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试删除不存在的配置
    let response = client
        .delete(&format!("{}/common-configs/99999", base_url))
        .send()
        .await
        .expect("删除不存在配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析删除不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试验证不存在的配置
    let response = client
        .get(&format!("{}/common-configs/99999/validate", base_url))
        .send()
        .await
        .expect("验证不存在配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析验证不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());

    // 测试根据key获取不存在的配置
    let response = client
        .get(&format!("{}/common-configs/key/nonexistent.key", base_url))
        .send()
        .await
        .expect("根据key获取不存在配置请求失败");

    assert_eq!(response.status(), 404);
    let data: Value = response.json().await.expect("解析根据key获取不存在错误响应失败");
    assert!(!data["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_common_config_batch_operations() {
    let base_url = "http://localhost:8080/api/v1";
    let client = reqwest::Client::new();

    // 创建多个配置用于批量操作测试
    let mut config_ids = Vec::new();

    for i in 1..=5 {
        let create_request = json!({
            "key": format!("batch.test.{}", i),
            "value": format!("value_{}", i),
            "description": format!("批量测试配置{}", i),
            "category": "批量测试",
            "data_type": "string",
            "is_encrypted": 0,
            "enabled": 1
        });

        let create_response = client
            .post(&format!("{}/common-configs", base_url))
            .json(&create_request)
            .send()
            .await
            .expect("创建批量测试配置请求失败");

        assert_eq!(create_response.status(), 200);
        let create_data: Value = create_response.json().await.expect("解析批量创建响应失败");
        config_ids.push(create_data["data"]["id"].as_i64().unwrap());
    }

    // 测试批量更新
    let batch_request = json!({
        "configs": [
            {
                "id": config_ids[0],
                "value": "updated_value_1",
                "enabled": 0
            },
            {
                "id": config_ids[1],
                "value": "updated_value_2",
                "description": "更新后的描述"
            },
            {
                "id": config_ids[2],
                "value": "updated_value_3"
            }
        ]
    });

    let batch_response = client
        .post(&format!("{}/common-configs/batch", base_url))
        .json(&batch_request)
        .send()
        .await
        .expect("批量更新请求失败");

    assert_eq!(batch_response.status(), 200);
    let batch_data: Value = batch_response.json().await.expect("解析批量更新响应失败");
    assert!(batch_data["success"].as_bool().unwrap());
    let updated_configs = batch_data["data"]["updated"].as_array().unwrap();
    assert_eq!(updated_configs.len(), 3);

    // 验证批量更新结果
    for &id in &config_ids[..3] {
        let verify_response = client
            .get(&format!("{}/common-configs/{}", base_url, id))
            .send()
            .await
            .expect("验证批量更新结果请求失败");

        assert_eq!(verify_response.status(), 200);
        let verify_data: Value = verify_response.json().await.expect("解析验证批量更新响应失败");
        assert!(verify_data["success"].as_bool().unwrap());
    }

    // 清理所有测试配置
    for &id in &config_ids {
        let delete_response = client
            .delete(&format!("{}/common-configs/{}", base_url, id))
            .send()
            .await
            .expect("清理批量测试配置请求失败");
        assert_eq!(delete_response.status(), 200);
    }
}
