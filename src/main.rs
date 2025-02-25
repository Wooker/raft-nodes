#![allow(unused)]
#![allow(dead_code)]

use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
    process::exit,
    str::FromStr,
};

mod node;
use node::Node;

mod peer;
use peer::Peer;

mod flow_entry;

const NODES: [SocketAddrV4; 2] = [
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5001),
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5002),
    // SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5003),
    // SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5004),
];

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Provide socket address as command line argument");
        exit(1);
    }
    let node = Node::new(
        0,
        SocketAddrV4::from_str(args[1].as_str()).unwrap(),
        args[2]
            .as_str()
            .parse()
            .expect("Could not parse controller port"),
        NODES.to_vec(),
    );
    let delay = node.delay;
    let mut peer = Peer::new(node, delay);
    peer.run().await
}
