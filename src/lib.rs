use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream, client_async};
use std::net::{IpAddr};
use shiplift::{Docker, ContainerOptions};
use futures::{StreamExt};
use futures::join;
use tokio::net::UnixStream;

pub struct DockerWebConsoleServer {
  host: IpAddr,
  port: u32,
  image: String
}

impl DockerWebConsoleServer {

  pub fn create<S: Into<String>>(host: IpAddr, port: u32, image: S) -> Self {
    Self { host, port, image: image.into() }
  }

  pub async fn start(self) {
    let address = format!("{}:{}", self.host.to_string(), self.port);
    println!("Starting server on {}...", address);
    
    let server = TcpListener::bind(address).await.unwrap();

    println!("Server successfully started.");

    loop {
      let (socket, _) = server.accept().await.unwrap();
      let image = self.image.to_string();
      tokio::spawn(async {
        let websocket = accept_async(socket).await.expect("Error during the websocket handshake occurred");

        println!("Client successfully connected.");
              
        let session = DockerWebConsoleServerSession::init(image, websocket).await;
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
  container_id: String,
  websocket: WebSocketStream<TcpStream>
}

impl DockerWebConsoleServerSession {

  async fn init(image: String, websocket: WebSocketStream<TcpStream>) -> Result<Self, shiplift::Error> {
    let docker = Docker::new();

    return match docker
        .containers()
        .create(&ContainerOptions::builder(image.as_ref()).tty(true).open_stdin(true).build())
        .await
    {
        Ok(info) => { 
          let handle = Self { container_id: info.id, websocket };
          Ok(handle)
       },
        Err(e) => { 
          eprintln!("Error: {}", e);
          Err(e)
        }
    }
  
  }

  async fn start(self) {
    let docker = Docker::new();
    
    let container_id = &self.container_id;
    let websocket = self.websocket;

    let (write_ws, read_ws) = websocket.split();

    match docker.containers().get(&self.container_id).start().await {
      Ok(_) => {
        println!("Container started.");
        
        let stream = UnixStream::connect("/var/run/docker.sock").await.unwrap();
        let (container_stream, _) = client_async(format!("ws://localhost/containers/{}/attach/ws?logs=1&stream=1&stdin=1&stdout=1&stderr=1", container_id), stream).await.expect("Failed to connect");
        let (write_c, read_c) = container_stream.split();
    
        let _ = join!(read_c.forward(write_ws), read_ws.forward(write_c));
      },
      Err(e) => {
        eprintln!("Error: {}", e);
      }
    }
  }

}

