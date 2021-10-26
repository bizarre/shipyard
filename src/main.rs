mod lib;
use structopt::StructOpt;
use lib::DockerWebConsoleServer;
use structopt_flags::{HostOpt};
use std::net::{IpAddr,Ipv4Addr};
use structopt_flags::GetWithDefault;

#[derive(Debug, StructOpt)]
#[structopt(name = "dwcs")]
struct Opt {
  image: String,
  #[structopt(flatten)]
  host: HostOpt,
  #[structopt(short, long, default_value = "8080")]
  port: u32,
}

#[tokio::main]
async fn main() {
  let opt = Opt::from_args();
  let host = opt.host.get_with_default(IpAddr::V4(Ipv4Addr::new(127,0,0,1)));

  let server = DockerWebConsoleServer::create(host, opt.port);
  server.start().await
}
