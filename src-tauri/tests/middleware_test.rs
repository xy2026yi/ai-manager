// 中间件和错误处理系统测试
//
// 测试请求追踪、认证中间件和错误处理功能

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use migration_ai_manager_lib::{
    api::error::ApiError,
    api::middleware::{add_request_id_header, global_error_handler, request_tracking_middleware},
};
use tower::ServiceExt;

fn create_test_app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route(
            "/error",
            get(|| async {
                ApiError::BadRequest { message: "测试错误".to_string() }.into_response()
            }),
        )
        .layer(middleware::from_fn(global_error_handler))
        .layer(middleware::from_fn(add_request_id_header))
        .layer(middleware::from_fn(request_tracking_middleware))
}

#[tokio::test]
async fn test_request_tracking_middleware() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/")
        .method(Method::GET)
        .header("user-agent", "test-agent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 检查是否添加了请求ID头
    let request_id_header = response.headers().get("x-request-id");
    assert!(request_id_header.is_some());
    println!("Request ID: {:?}", request_id_header);
}

#[tokio::test]
async fn test_error_handling() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/error")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 检查是否添加了请求ID头
    let request_id_header = response.headers().get("x-request-id");
    assert!(request_id_header.is_some());
}

// 简化测试，专注于中间件的基本功能
#[tokio::test]
async fn test_middleware_integration() {
    // 测试多个中间件的集成
    let app = Router::new()
        .route("/", get(|| async { "Middleware Test Success" }))
        .layer(middleware::from_fn(global_error_handler))
        .layer(middleware::from_fn(add_request_id_header))
        .layer(middleware::from_fn(request_tracking_middleware));

    let request = Request::builder()
        .uri("/")
        .method(Method::GET)
        .header("user-agent", "test-integration")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 检查响应头
    let request_id = response.headers().get("x-request-id");
    assert!(request_id.is_some(), "应该包含请求ID头");

    // 可以手动检查响应内容
    // let body_bytes = axum::body::to_bytes(response.into_body(), hyper::body::SizeHint::default()).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert_eq!(body_str, "Middleware Test Success");
}
