use libp2p::{
    ping,
    request_response::{Behaviour, Codec, Event, ProtocolSupport},
    swarm::NetworkBehaviour,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MessageProtocolEvent")]
pub struct MessageProtocol {
    request_response: Behaviour<FileExchangeCodec>,
    ping: ping::Behaviour,
}

impl MessageProtocol {
    pub fn new_with_ping(ping_interval: Option<std::time::Duration>) -> Self {
        let protocol = Behaviour::new(FileExchangeCodec, ProtocolSupport::Full);
        let mut cfg = ping::Config::default();
        if let Some(int) = ping_interval {
            cfg = cfg.with_interval(int);
        }
        let ping_behaviour = ping::Behaviour::new(cfg);
        Self { request_response: protocol, ping: ping_behaviour }
    }
}

#[derive(Debug)]
pub enum MessageProtocolEvent {
    RequestResponse(Event<MessageRequest, MessageResponse>),
    Ping(ping::Event),
}

impl From<Event<MessageRequest, MessageResponse>> for MessageProtocolEvent {
    fn from(event: Event<MessageRequest, MessageResponse>) -> Self {
        MessageProtocolEvent::RequestResponse(event)
    }
}

impl From<ping::Event> for MessageProtocolEvent {
    fn from(event: ping::Event) -> Self {
        MessageProtocolEvent::Ping(event)
    }
}

#[derive(Debug, Clone)]
pub struct FileExchangeCodec;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageRequest {
    pub message_id: uuid::Uuid,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResponse {
    pub received: bool,
}

impl Codec for FileExchangeCodec {
    type Protocol = String;
    type Request = MessageRequest;
    type Response = MessageResponse;

    fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<Self::Request>> + Send + '_>>
    where
        T: tokio::io::AsyncRead + Unpin + Send,
    {
        Box::pin(async move {
            let mut len_bytes = [0u8; 4];
            io.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;
            let mut buf = vec![0u8; len];
            io.read_exact(&mut buf).await?;
            bincode::deserialize(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send + '_>>
    where
        T: tokio::io::AsyncWrite + Unpin + Send,
    {
        Box::pin(async move {
            let bytes = bincode::serialize(&req)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let len = bytes.len() as u32;
            io.write_all(&len.to_be_bytes()).await?;
            io.write_all(&bytes).await?;
            io.flush().await
        })
    }

    fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<Self::Response>> + Send + '_>>
    where
        T: tokio::io::AsyncRead + Unpin + Send,
    {
        Box::pin(async move {
            let mut len_bytes = [0u8; 4];
            io.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;
            let mut buf = vec![0u8; len];
            io.read_exact(&mut buf).await?;
            bincode::deserialize(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, resp: Self::Response) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send + '_>>
    where
        T: tokio::io::AsyncWrite + Unpin + Send,
    {
        Box::pin(async move {
            let bytes = bincode::serialize(&resp)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let len = bytes.len() as u32;
            io.write_all(&len.to_be_bytes()).await?;
            io.write_all(&bytes).await?;
            io.flush().await
        })
    }
}