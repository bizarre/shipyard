use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use tungstenite::{WebSocket, accept};
use std::net::{IpAddr};

pub struct DockerWebConsoleServer {
  host: IpAddr,
  port: u32
}

impl DockerWebConsoleServer {

  pub fn create(host: IpAddr, port: u32) -> Self {
    Self { host, port }
  }

  pub fn start(self) {
    let address = format!("{}:{}", self.host.to_string(), self.port);
    println!("Starting server on {}...", address);
    
    let server = TcpListener::bind(address).unwrap();

    println!("Server successfully started.");

    for stream in server.incoming() {
      spawn (move || {
          let websocket = accept(stream.unwrap()).unwrap();
          
          println!("Client successfully connected.");
          
          let session = DockerWebConsoleServerSession::init(websocket);
          
          if let Ok(session) = session {
            session.start()
          } else {
            println!("Failed to setup docker container.")
          }
      });
    }

  }
}

struct DockerWebConsoleServerSession<T> {
  websocket: WebSocket<T>
}

impl <T> DockerWebConsoleServerSession<T> where T: Write + Read {

  fn init(websocket: WebSocket<T>) -> std::io::Result<Self> {
    Ok(Self { websocket })
  }

  fn start(self) {
    let mut websocket = self.websocket;

    loop {
      let msg = websocket.read_message().unwrap();

      if msg.is_binary() || msg.is_text() {
          websocket.write_message(msg).unwrap();
      }
    }

  }

}
