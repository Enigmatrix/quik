use std::collections::HashMap;
use std::sync::RwLock;

use quik_util::*;

use crate::common::{ConnectionId, StreamId};
use crate::crypto::Crypto;
use crate::frame::Frame;
use crate::packet::Packet;
use crate::transport::Io;

pub struct Stream {
    
}

pub struct Connection {
    pub steams: HashMap<StreamId, Stream>
}

pub struct DefaultHandler {
    pub connections: RwLock<HashMap<ConnectionId, Connection>>
}

impl DefaultHandler {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new())
        }
    }
}

impl Handler for DefaultHandler  {
    async fn handle<'a>(
        &self,
        packet: Packet<'a>,
        frames: impl Iterator<Item = Result<Frame<'a>>>,
    ) -> Result<()> {
        // TODO: why do i need a unwrap here?
        let conns = self.connections.read().unwrap();
        if let Some(conn) = conns.get(packet.dst_cid()) {

        } else {
            drop(conns);
            self.connections.write().unwrap().insert(packet.dst_cid().clone(), Connection {
                steams: HashMap::new()
            });
        }

        todo!()
    }
}

// if this needs to be mutable inside, then it should use a mutex internally
pub trait Handler {
    fn handle<'a>(
        &self,
        packet: Packet<'a>,
        frames: impl Iterator<Item = Result<Frame<'a>>>,
    ) -> impl Future<Output = Result<()>>;
}

pub trait Server {
    type Crypto: Crypto;
    // UDP transport
    type Io: Io;
    type Handler: Handler;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct DefaultCrypto;
    impl Crypto for DefaultCrypto {
        async fn decrypt_initial_data(
            &self,
            cid: ConnectionId,
            version: u32,
            is_server: bool,
            data: &mut impl Buffer,
        ) -> Result<Vec<u8>> {
            todo!()
        }
    }
    
    struct DefaultIo;
    impl Io for DefaultIo {
        async fn send(&self, data: &[u8]) -> Result<()> {
            todo!()
        }
        async fn recv(&self, data: &mut [u8]) -> Result<()> {
            todo!()
        }
        async fn close(self) -> () {
            todo!()
        }
    }
    
    struct DefaultHandler<P: Provider> {
        provider: P,
    }
    impl<P: Provider> Handler for DefaultHandler<P> {
        async fn handle<'a>(
            &self,
            packet: Packet<'a>,
            frames: impl Iterator<Item = Result<Frame<'a>>>,
        ) -> Result<()> {
            todo!()
        }
    }
    impl<P: Provider> DefaultHandler<P> {
        pub fn new(provider: P) -> Self {
            Self { provider }
        }
    }
    
    struct DefaultServer<P: Provider> {
        handler: DefaultHandler<P>,
    }
    impl<P: Provider> Server for DefaultServer<P> {
        type Crypto = DefaultCrypto;
        type Io = DefaultIo;
        type Handler = DefaultHandler<P>;
    }
    impl<P: Provider> DefaultServer<P> {
        pub fn new(handler: <Self as Server>::Handler) -> Self {
            todo!()
        }

        pub async fn run(&mut self) -> Result<()> {
            todo!()
        }
    }

    trait Stream {
        fn recv(&self, data: &[u8]) -> impl Future<Output = Result<()>>;
        fn dropped(&self) -> impl Future<Output = Result<()>>;
    }
    
    trait Connection {
        type Stream: Stream;

        fn on_stream(&self, id: StreamId) -> impl Future<Output = Result<Self::Stream>>;
        fn dropped(&self) -> impl Future<Output = Result<()>>;
    }
    
    trait Provider {
        type Connection: Connection;
    
        fn on_connect(&self, id: ConnectionId) -> impl Future<Output = Result<Self::Connection>>;
    }
    
    #[tokio::test]
    async fn test_example() -> Result<()> {
        // let handler = DefaultHandler::new();
        // handler.on_connect(|conn| async {
        //     conn.on_stream(|stream| async {
        //         stream.on_data(|data| async {
        //             println!("Received data: {:?}", data);
        //             Ok(())
        //         }).await?;

        //         stream.on_close(|| async {
        //             println!("Stream closed");
        //             Ok(())
        //         }).await?;

        //         Ok(())
        //     }).await?;

        //     conn.on_close(|| async {
        //         println!("Connection closed");
        //         Ok(())
        //     }).await?;

        //     Ok(())
        // });
        
        
        let handler = ConnectProvider::new(|conn_id| async {
            Ok(Connection::new())
        });
        
        DefaultServer::new(handler).run().await?;
        Ok(())
    }
}