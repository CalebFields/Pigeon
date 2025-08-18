use secure_p2p_msg::api::Core;

#[tokio::test]
async fn check_for_update_handles_non_200() {
    let core = Core::default();
    // Likely returns 404 for root path without trailing slash
    let out = core.check_for_update("https://example.com/not-found").await.unwrap();
    assert!(out.is_none());
}


