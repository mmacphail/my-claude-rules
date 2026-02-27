mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_create_item() {
    let app = common::TestApp::spawn().await;

    let res = app.server
        .post("/api/v1/items")
        .json(&serde_json::json!({ "name": "Test Item" }))
        .await;

    res.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = res.json();
    assert_eq!(body["name"], "Test Item");

    app.cleanup().await;
}

#[tokio::test]
async fn test_list_items() {
    let app = common::TestApp::spawn().await;

    app.server
        .post("/api/v1/items")
        .json(&serde_json::json!({ "name": "Item A" }))
        .await;

    let res = app.server.get("/api/v1/items").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["data"].as_array().unwrap().len() >= 1);

    app.cleanup().await;
}

#[tokio::test]
async fn test_get_item_not_found() {
    let app = common::TestApp::spawn().await;

    let res = app.server
        .get("/api/v1/items/00000000-0000-0000-0000-000000000000")
        .await;
    res.assert_status_not_found();

    app.cleanup().await;
}
