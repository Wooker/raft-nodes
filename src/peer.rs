use std::{
    io::{Read, Write},
    net::{SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, Mutex, RwLock},
    thread::{self, sleep},
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
        // Thread for handling incoming connections
        let node_l = Arc::clone(&self.node);
        thread::spawn(move || {
            println!("Listening");

            // Get address outside of incoming loop
            let addr = {
                let node_read = node_l.read().unwrap();
                node_read.addr
            };

            let listener = TcpListener::bind(addr).unwrap();
            for s in listener.incoming() {
                println!();
                match s {
                    Ok(mut stream) => {
                        let role = {
                            let node_read = node_l.read().unwrap();
                            node_read.role
                        };
                        println!("{:?}", role);

                        // Handling different roles
                        match role {
                            Role::Leader => {
                                // Leader-specific logic here
                            }
                            Role::Follower => {
                                // Follower-specific logic here
                                let mut buf = String::new();
                                stream.read_to_string(&mut buf).unwrap();
                                let message =
                                    bincode::deserialize::<Message>(&buf.as_bytes()).unwrap();
                                println!("Message: {:?}", message);

                                let response = match message {
                                    Message::RequestVote { candidate_id } => {
                                        Message::Vote { candidate_id }
                                    }
                                    Message::AppendEntries {
                                        entries,
                                        leader_term,
                                    } => todo!(),
                                    Message::Vote { candidate_id } => todo!(),
                                };
                                println!("Response: {:?}", response);
                                stream
                                    .write(
                                        bincode::serialize(&response)
                                            .expect("Could not serialize response")
                                            .as_slice(),
                                    )
                                    .unwrap();
                            }
                            Role::Candidate => {
                                // Read incoming data outside of lock scope
                                let mut buf = String::new();
                                stream.read_to_string(&mut buf).unwrap();

                                let message =
                                    bincode::deserialize::<Message>(&buf.as_bytes()).unwrap();
                                println!("Message: {:?}", message);

                                let response = match message {
                                    Message::RequestVote { candidate_id } => {
                                        Message::Vote { candidate_id }
                                    }
                                    Message::AppendEntries {
                                        entries,
                                        leader_term,
                                    } => todo!(),
                                    Message::Vote { candidate_id } => todo!(),
                                };

                                println!("Response: {:?}", response);
                                stream
                                    .write(
                                        bincode::serialize(&response)
                                            .expect("Could not serialize response")
                                            .as_slice(),
                                    )
                                    .unwrap();

                                // Only acquire a write lock when needed
                                let mut node_write = node_l.write().unwrap();
                                node_write.role = Role::Follower;

                                // Flush and finalize handling
                                stream.flush().unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        });

        // Thread for periodic tasks based on role
        let delay = self.delay;
        let node_s = Arc::clone(&self.node);
        thread::spawn(move || loop {
            sleep(delay);
            println!();

            // Read role to avoid locking node for entire loop duration
            let role = {
                let node_read = node_s.read().unwrap();
                node_read.role
            };
            println!("{:?}", role);

            let id = {
                let node_read = node_s.read().unwrap();
                node_read.id
            };

            match role {
                Role::Leader => {
                    // Implement leader-specific periodic task
                    println!("Leader");
                }
                Role::Follower => {
                    // Follower-specific task loop
                    println!("Follower");
                    loop {
                        sleep(delay);
                    }
                }
                Role::Candidate => {
                    // Read addresses outside loop to avoid re-locking
                    let addresses: Vec<SocketAddrV4> = {
                        let node_read = node_s.read().unwrap();
                        node_read
                            .nodes
                            .iter()
                            .filter(|&&p| p != node_read.addr)
                            .cloned()
                            .collect()
                    };

                    for addr in addresses {
                        if let Ok(mut stream) =
                            TcpStream::connect_timeout(&addr.into(), Duration::from_secs(5))
                        {
                            // Sending data or additional candidate logic here
                            stream
                                .write(
                                    bincode::serialize(&Message::RequestVote { candidate_id: id })
                                        .unwrap()
                                        .as_slice(),
                                )
                                .unwrap();
                        }
                    }
                }
            }
        });

        // Main thread loop
        loop {}
    }
}
