use std::time::Duration;

#[tokio::test]
async fn test_health_endpoint_no_auth() {
    let port = portpicker::pick_unused_port().expect("no free port");
    let config = test_config(port, None);

    let handle = tokio::spawn(async move {
        origin_mcp::serve::run_serve(config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let resp = reqwest::get(format!("http://127.0.0.1:{}/health", port))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["server"], "origin-mcp");

    handle.abort();
}

#[tokio::test]
async fn test_auth_rejects_missing_token() {
    let port = portpicker::pick_unused_port().expect("no free port");
    let config = test_config(port, Some("secret-token".into()));

    let handle = tokio::spawn(async move {
        origin_mcp::serve::run_serve(config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);

    handle.abort();
}

#[tokio::test]
async fn test_auth_rejects_wrong_token() {
    let port = portpicker::pick_unused_port().expect("no free port");
    let config = test_config(port, Some("correct-token".into()));

    let handle = tokio::spawn(async move {
        origin_mcp::serve::run_serve(config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer wrong-token")
        .body("{}")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);

    handle.abort();
}

#[tokio::test]
async fn test_health_bypasses_auth() {
    let port = portpicker::pick_unused_port().expect("no free port");
    let config = test_config(port, Some("secret-token".into()));

    let handle = tokio::spawn(async move {
        origin_mcp::serve::run_serve(config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let resp = reqwest::get(format!("http://127.0.0.1:{}/health", port))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    handle.abort();
}

#[tokio::test]
async fn test_rejects_disallowed_origin() {
    let port = portpicker::pick_unused_port().expect("no free port");
    let config = origin_mcp::serve::ServeConfig {
        port,
        host: "127.0.0.1".into(),
        origin_url: "http://127.0.0.1:19999".into(),
        token: Some("test-token".into()),
        agent_name: "test-agent".into(),
        user_id: None,
        allowed_origins: vec!["https://claude.ai".into()],
    };

    let handle = tokio::spawn(async move {
        origin_mcp::serve::run_serve(config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();

    // Request with disallowed Origin should get 403
    let resp = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("Origin", "https://evil.com")
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // Request with allowed Origin should pass auth (may fail downstream, but not 403)
    let resp = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test-token")
        .header("Origin", "https://claude.ai")
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 403);

    handle.abort();
}

fn test_config(port: u16, token: Option<String>) -> origin_mcp::serve::ServeConfig {
    origin_mcp::serve::ServeConfig {
        port,
        host: "127.0.0.1".into(),
        origin_url: "http://127.0.0.1:19999".into(), // non-existent, OK for these tests
        token,
        agent_name: "test-agent".into(),
        user_id: None,
        allowed_origins: vec!["*".into()],
    }
}
