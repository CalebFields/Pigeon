#[test]
fn metrics_render_increments() {
    let m = secure_p2p_msg::ops::Metrics::default();
    m.sent_messages
        .fetch_add(2, std::sync::atomic::Ordering::Relaxed);
    m.delivered_messages
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    m.failed_messages
        .fetch_add(3, std::sync::atomic::Ordering::Relaxed);
    m.received_messages
        .fetch_add(4, std::sync::atomic::Ordering::Relaxed);

    let out = m.render_prometheus();
    assert!(out.contains("pigeon_sent_messages 2"));
    assert!(out.contains("pigeon_delivered_messages 1"));
    assert!(out.contains("pigeon_failed_messages 3"));
    assert!(out.contains("pigeon_received_messages 4"));
}
