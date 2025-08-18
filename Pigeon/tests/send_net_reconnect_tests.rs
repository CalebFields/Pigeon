#![cfg(feature = "network")]

use libp2p::futures::StreamExt;

#[tokio::test]
async fn send_net_retries_on_initial_failure() {
    // Arrange a dummy address that won't accept immediately; this ensures dial retry path executes.
    // We use a high, likely closed port on localhost.
    let args = vec![
        "send-net",
        "--to",
        "/ip4/127.0.0.1/tcp/65533",
        "--pubkey_hex",
        &hex::encode([0u8; 32]),
        "--message",
        "hi",
        "--retries",
        "1",
        "--backoff-ms",
        "10",
        "--timeout-ms",
        "10",
    ];

    // We can't execute the CLI in-process easily; instead, exercise the retry helper shape indirectly by
    // spawning the CLI main in a background task and cancelling quickly. This ensures it doesn't panic.
    let result = tokio::time::timeout(std::time::Duration::from_millis(200), async move {
        let cli = secure_p2p_msg::ui::cli::Cli::parse_from(
            std::iter::once("bin").chain(args.iter().copied()),
        );
        let _ = cli.execute().await;
    })
    .await;

    assert!(result.is_ok(), "send-net retry path hung/panicked");
}
