use std::io::{Read, Write};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite};
use std::thread::spawn;
use tokio_tungstenite::{accept_async, WebSocketStream};
use std::net::{IpAddr};
use shiplift::{tty::TtyChunk, Docker};
use futures::future;
use futures::{Stream, StreamExt};
use futures::prelude::*;

pub struct DockerWebConsoleServer {
  host: IpAddr,
  port: u32
}

impl DockerWebConsoleServer {

  pub fn create(host: IpAddr, port: u32) -> Self {
    Self { host, port }
  }

  pub async fn start(self) {
    let address = format!("{}:{}", self.host.to_string(), self.port);
    println!("Starting server on {}...", address);
    
    let server = TcpListener::bind(address).await.unwrap();

    println!("Server successfully started.");

    loop {
      let (socket, _) = server.accept().await.unwrap();
      tokio::spawn(async {
        let websocket = accept_async(socket).await.expect("Error during the websocket handshake occurred");

        println!("Client successfully connected.");
              
        let session = DockerWebConsoleServerSession::init(websocket);
        if let Ok(session) = session {
          session.start().await
        } else {
          println!("Failed to start docker container.")
        }
      });
    }

  }
}

struct DockerWebConsoleServerSession {
  websocket: WebSocketStream<TcpStream>
}

impl DockerWebConsoleServerSession {

  fn init(websocket: WebSocketStream<TcpStream>) -> std::io::Result<Self> {
    let docker = Docker::new();
    // prep
    Ok(Self { websocket })
  }

  async fn start(self) {
    let  websocket = self.websocket;
    let (write, read) = websocket.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
  }

}
