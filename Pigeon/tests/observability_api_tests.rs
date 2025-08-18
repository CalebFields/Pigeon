use secure_p2p_msg::api::Core;

#[tokio::test]
async fn ops_server_starts_and_serves_metrics() {
    let core = Core::default();
    // Bind to an ephemeral localhost port
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let local = listener.local_addr().unwrap();
    drop(listener);

    let handle = core.start_ops_server(local);
    // Give server a moment
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let resp = reqwest::get(format!("http://{}", local))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(resp.contains("pigeon_sent_messages"));
    handle.abort();
}


