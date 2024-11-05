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

const NODES: [SocketAddrV4; 4] = [
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5001),
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5002),
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5003),
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5004),
];

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Args: {:?}", args);
    if args.len() != 2 {
        println!("Provide socket address as command line argument");
        exit(1);
    }
    let node = Node::new(
        0,
        SocketAddrV4::from_str(args[1].as_str()).unwrap(),
        NODES.to_vec(),
    );
    let delay = node.delay;
    let peer = Peer::new(node, delay);
    peer.run();
}
