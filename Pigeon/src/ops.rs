use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};

#[derive(Default, Clone)]
pub struct Metrics {
    pub sent_messages: Arc<AtomicU64>,
    pub delivered_messages: Arc<AtomicU64>,
    pub failed_messages: Arc<AtomicU64>,
    pub received_messages: Arc<AtomicU64>,
}

impl Metrics {
    pub fn render_prometheus(&self) -> String {
        format!(
            concat!(
                "# HELP pigeon_sent_messages Total messages attempted to send\n",
                "# TYPE pigeon_sent_messages counter\n",
                "pigeon_sent_messages {}\n",
                "# HELP pigeon_delivered_messages Total messages acknowledged as delivered\n",
                "# TYPE pigeon_delivered_messages counter\n",
                "pigeon_delivered_messages {}\n",
                "# HELP pigeon_failed_messages Total send failures\n",
                "# TYPE pigeon_failed_messages counter\n",
                "pigeon_failed_messages {}\n",
                "# HELP pigeon_received_messages Total messages received\n",
                "# TYPE pigeon_received_messages counter\n",
                "pigeon_received_messages {}\n",
            ),
            self.sent_messages.load(Ordering::Relaxed),
            self.delivered_messages.load(Ordering::Relaxed),
            self.failed_messages.load(Ordering::Relaxed),
            self.received_messages.load(Ordering::Relaxed),
        )
    }
}

pub async fn serve(addr: SocketAddr, metrics: Metrics) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let m = metrics.clone();
        tokio::spawn(async move {
            let mut sock = socket;
            // Read a little from the client (best-effort) so HTTP clients don't error on early close
            let mut buf = [0u8; 1024];
            let _ = timeout(Duration::from_millis(250), sock.read(&mut buf)).await;

            let body = m.render_prometheus();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        });
    }
}
