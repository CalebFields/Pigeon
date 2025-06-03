use libp2p::{
    request_response::{ProtocolSupport, RequestResponse, RequestResponseConfig, RequestResponseEvent},
    swarm::NetworkBehaviour,
    PeerId,
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MessageProtocolEvent")]
pub struct MessageProtocol {
    request_response: RequestResponse<FileExchangeCodec>,
}

impl MessageProtocol {
    pub fn new() -> Self {
        let protocol = RequestResponse::new(
            FileExchangeCodec,
            ProtocolSupport::Full,
            RequestResponseConfig::default(),
        );
        Self {
            request_response: protocol,
        }
    }
}

#[derive(Debug)]
pub enum MessageProtocolEvent {
    RequestResponse(RequestResponseEvent<MessageRequest, MessageResponse>),
}

impl From<RequestResponseEvent<MessageRequest, MessageResponse>> for MessageProtocolEvent {
    fn from(event: RequestResponseEvent<MessageRequest, MessageResponse>) -> Self {
        MessageProtocolEvent::RequestResponse(event)
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

impl libp2p::request_response::Codec for FileExchangeCodec {
    type Protocol = String;
    type Request = MessageRequest;
    type Response = MessageResponse;

    fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Request>
    where
        T: tokio::io::AsyncRead + Unpin + Send,
    {
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: &Self::Request) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWrite + Unpin + Send,
    {
        let bytes = bincode::serialize(req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let len = bytes.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&bytes).await?;
        io.flush().await
    }

    // Similar implementations for read_response/write_response
    // ...
}