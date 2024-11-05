use std::{
    io::{Read, Write},
    net::{SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread::{self, park_timeout, sleep},
    time::Duration,
};

use crate::node::{Message, Node, Role};

pub struct Peer {
    node: Arc<RwLock<Node>>,
    delay: Duration,
}

impl Peer {
    pub fn new(node: Node, delay: Duration) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
            delay,
        }
    }

    pub fn run(&self) {
        // Spawn a thread for handling incoming connections
        let node_l = Arc::clone(&self.node);
        thread::spawn(move || {
            println!("Listening");

            let addr = {
                let node_read = node_l.read().unwrap();
                node_read.addr
            };

            let listener = TcpListener::bind(addr).unwrap();
            for s in listener.incoming() {
                match s {
                    Ok(mut stream) => {
                        let role = {
                            let node_read = node_l.read().unwrap();
                            node_read.role
                        };
                        match role {
                            Role::Leader => Self::handle_leader(&node_l, &mut stream),
                            Role::Follower => Self::handle_follower(&node_l, &mut stream),
                            Role::Candidate => Self::handle_candidate(&node_l, &mut stream),
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        });

        // Spawn a thread for periodic tasks based on the role
        let delay = self.delay;
        let node_s = Arc::clone(&self.node);
        thread::spawn(move || loop {
            sleep(delay);

            let role = {
                let node_read = node_s.read().unwrap();
                node_read.role
            };

            match role {
                Role::Leader => Self::leader_periodic_task(&node_s),
                Role::Follower => Self::follower_periodic_task(&node_s, delay),
                Role::Candidate => Self::candidate_periodic_task(&node_s),
            }
        });

        loop {}
    }

    fn handle_leader(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        println!("Leader handling incoming connection");
        // Leader-specific logic here
    }

    fn handle_follower(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        // Follower-specific logic here
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        let message = bincode::deserialize::<Message>(buf.as_bytes()).unwrap();

        println!("\nFollower: {:?}", message);

        let response = match message {
            Message::RequestVote { candidate_id } => Message::Vote { candidate_id },
            Message::Vote { candidate_id } => todo!(),
            Message::AppendEntries {
                entries,
                leader_term,
            } => todo!(),
            Message::RequestEntries { candidate_id } => todo!(),
        };

        println!("Follower reply: {:?}", response);
        stream
            .write_all(&bincode::serialize(&response).unwrap())
            .unwrap();
        stream.flush().unwrap();
    }

    fn handle_candidate(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        println!("Candidate handling incoming connection");

        // Read data outside of lock
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();

        // Only acquire write lock when needed
        let mut node_write = node.write().unwrap();
        node_write.role = Role::Follower;
        println!("Now Follower");

        // Flush and finalize handling
        stream.flush().unwrap();
    }

    fn leader_periodic_task(node: &Arc<RwLock<Node>>) {
        println!("Leader periodic task");
        // Implement leader-specific periodic task
    }

    fn follower_periodic_task(node: &Arc<RwLock<Node>>, delay: Duration) {
        println!("Follower periodic task");

        // Implement follower-specific task loop
        loop {
            sleep(delay);
        }
    }

    fn candidate_periodic_task(node: &Arc<RwLock<Node>>) {
        println!("Candidate periodic task");

        let addresses: Vec<SocketAddrV4> = {
            let node_read = node.read().unwrap();
            node_read
                .nodes
                .iter()
                .filter(|&&p| p != node_read.addr)
                .cloned()
                .collect()
        };

        for addr in addresses {
            if let Ok(mut stream) = TcpStream::connect_timeout(&addr.into(), Duration::from_secs(5))
            {
                println!("Connected to: {}", addr);
                stream
                    .write_all(
                        bincode::serialize(&Message::RequestVote {
                            candidate_id: node.read().unwrap().id,
                        })
                        .unwrap()
                        .as_slice(),
                    )
                    .unwrap();

                // Read response
                let mut buf = Vec::new();
                if stream.read_to_end(&mut buf).is_ok() {
                    if let Ok(response) = bincode::deserialize::<Message>(&buf) {
                        println!("Candidate received response: {:?}", response);
                    } else {
                        println!("Failed to deserialize response from {:?}", addr);
                    }
                } else {
                    println!("Failed to read response from {:?}", addr);
                }
            }
        }
    }
}
