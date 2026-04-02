//! Direct peer-to-peer messaging protocol for SAGE.
//!
//! Implements `libp2p::request_response::Codec` so SAGE nodes can send
//! `GossipMessage` directly to a specific peer over a dedicated substream,
//! rather than broadcasting over GossipSub.
//!
//! Protocol ID: `/sage/direct/1.0.0`
//! Framing: 4-byte big-endian length prefix, then raw bytes (max 1 MiB).
//! Response: empty ACK (fire-and-forget semantics).

use async_trait::async_trait;
use futures::prelude::*;
use libp2p::request_response;
use std::io;

// ─── Protocol ────────────────────────────────────────────────────────────────

/// Protocol identifier for versioned negotiation.
#[derive(Debug, Clone)]
pub struct SageDirectProtocol;

impl AsRef<str> for SageDirectProtocol {
    fn as_ref(&self) -> &str {
        "/sage/direct/1.0.0"
    }
}

pub use request_response::ProtocolSupport;

// ─── Message types ───────────────────────────────────────────────────────────

/// A direct request: the raw bincode bytes of a `GossipMessage`.
pub type DirectRequest = Vec<u8>;

/// An ACK response: empty vec (fire-and-forget).
pub type DirectResponse = Vec<u8>;

// ─── Codec ───────────────────────────────────────────────────────────────────

const MAX_FRAME: usize = 1024 * 1024; // 1 MiB

/// Length-prefixed bincode codec for direct SAGE messages.
#[derive(Debug, Clone, Default)]
pub struct SageDirectCodec;

#[async_trait]
impl request_response::Codec for SageDirectCodec {
    type Protocol = SageDirectProtocol;
    type Request = DirectRequest;
    type Response = DirectResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_length_prefixed(io).await
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_length_prefixed(io).await
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, req).await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, res).await
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn read_length_prefixed<T: AsyncRead + Unpin + Send>(io: &mut T) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame too large: {len} bytes (max {MAX_FRAME})"),
        ));
    }
    let mut data = vec![0u8; len];
    io.read_exact(&mut data).await?;
    Ok(data)
}

async fn write_length_prefixed<T: AsyncWrite + Unpin + Send>(
    io: &mut T,
    data: Vec<u8>,
) -> io::Result<()> {
    let len = (data.len() as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(&data).await?;
    io.flush().await?;
    Ok(())
}

// ─── Behaviour alias ─────────────────────────────────────────────────────────

/// Type alias for the request-response behaviour wired into `SageBehaviour`.
pub type DirectSendBehaviour = request_response::Behaviour<SageDirectCodec>;

/// Create a new `DirectSendBehaviour` with default config.
pub fn make_direct_send_behaviour() -> DirectSendBehaviour {
    request_response::Behaviour::new(
        vec![(SageDirectProtocol, ProtocolSupport::Full)],
        request_response::Config::default(),
    )
}
