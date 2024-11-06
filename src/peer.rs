use std::{
    cmp::max,
    io::{self},
    net::{SocketAddr, SocketAddrV4},
    sync::{Arc, RwLock},
    thread::{self, park_timeout, sleep},
    time::{Duration, Instant},
};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time,
};

use crate::node::{Message, Node, Role};

pub struct Peer {
    node: Arc<RwLock<Node>>,
    delay: Duration,
}

pub enum Action {
    NOP,
    Become(Role),
    AddVote,
    Append(Vec<String>),
}

impl Peer {
    pub fn new(node: Node, delay: Duration) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
            delay,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        loop {
            // Step 1: Run listener for a specific duration
            self.run_listener().await?;

            {
                println!("{}", self.node.read().unwrap().term);
            }
            {
                self.node.write().unwrap().role = Role::Candidate;
            }

            // Step 2: After listener duration, switch to client mode
            println!("Listener duration expired. Switching to client mode.");
            self.run_sender().await?;

            println!("Finished client connections. Returning.");
        }
        Ok(())
    }

    async fn run_listener(&mut self) -> io::Result<()> {
        let (addr, role, term) = {
            let node = self.node.read().unwrap();
            (node.addr, node.role, node.term)
        };

        'listener: loop {
            // Create a new TcpSocket for listening
            let sock = TcpSocket::new_v4().unwrap();
            sock.set_reuseaddr(true).unwrap();
            sock.bind(std::net::SocketAddr::V4(addr))
                .expect("Could not bind the socket for listening");

            let listener = sock.listen(1024).unwrap();

            let accept_timeout = self.delay;
            match time::timeout(accept_timeout, listener.accept()).await {
                Ok(Ok((stream, addr))) => {
                    let (stream, mut message) = Self::listener_read_message(stream, role, term)
                        .await
                        .unwrap();

                    match message {
                        Message::Vote { candidate_id, term } => {
                            self.node.write().unwrap().role = Role::Follower;
                            self.node.write().unwrap().term = term;
                        }
                        Message::AppendEntries {
                            ref mut entries,
                            term,
                        } => {
                            self.node.write().unwrap().log.append(entries);
                            self.node.write().unwrap().term = term
                        }
                        _ => todo!(),
                    }

                    Self::listener_write_message(stream, message).await.unwrap();
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to accept connection: {}", e);
                    break;
                }
                Err(_) => {
                    println!("No connection received within the timeout period");
                    drop(listener);
                    break 'listener;
                }
            }
            println!();
        }
        Ok(())
    }

    async fn run_sender(&self) -> io::Result<()> {
        'sender: loop {
            let (addr, nodes, role, term) = {
                let node = self.node.read().unwrap();
                let addr = node.addr.clone();
                (
                    addr,
                    node.nodes
                        .clone()
                        .into_iter()
                        .filter(|p| *p != addr)
                        .collect::<Vec<SocketAddrV4>>()
                        .clone(),
                    node.role,
                    node.term,
                )
            };

            for &node_addr in &nodes {
                // Create a new TcpSocket for each connection attempt
                let sock = TcpSocket::new_v4().unwrap();
                sock.set_reuseaddr(true).unwrap();
                sock.bind(SocketAddr::V4(addr))
                    .expect("Could not bind the socket for stream");

                match sock.connect(SocketAddr::V4(node_addr)).await {
                    Ok(mut stream) => {
                        // First write message then read the response
                        let action = Self::sender_read_message(
                            Self::sender_write_message(stream, role, term)
                                .await
                                .unwrap(),
                        )
                        .await
                        .unwrap();

                        match action {
                            Action::Become(r) => {
                                {
                                    self.node.write().unwrap().role = r;
                                }
                                let new_role = { self.node.read().unwrap().role };
                                println!("New role: {:?}", new_role);
                            }
                            Action::AddVote => self.node.write().unwrap().votes += 1,
                            Action::Append(mut vec) => {
                                self.node.write().unwrap().log.append(&mut vec)
                            }
                            Action::NOP => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect as a client: {}", e);
                    }
                }
            }

            println!();
            self.node.write().unwrap().term += 1;
            thread::sleep(self.delay - Duration::from_secs(1));
        }

        Ok(())
    }

    async fn sender_write_message(
        mut stream: TcpStream,
        role: Role,
        term: usize,
    ) -> tokio::io::Result<TcpStream> {
        let message = match role {
            Role::Leader => Message::RequestEntries {
                candidate_id: stream.local_addr().unwrap(),
                term,
            },
            Role::Follower => todo!(),
            Role::Candidate => Message::RequestVote {
                candidate_id: stream.local_addr().unwrap(),
                term,
            },
        };
        stream
            .write_all(bincode::serialize(&message).unwrap().as_slice())
            .await
            .unwrap();
        stream.flush().await.unwrap();
        println!(">>> {:?} [{}]", message, stream.peer_addr().unwrap());

        Ok(stream)
    }
    async fn sender_read_message(mut stream: TcpStream) -> tokio::io::Result<Action> {
        let mut buffer = [0; 1024];
        let action = match stream.read(&mut buffer).await {
            Ok(bytes_read) => {
                let message = bincode::deserialize::<Message>(&buffer).unwrap();
                println!("<<< {:?} [{}]", message, stream.peer_addr().unwrap());

                match message {
                    Message::Vote { candidate_id, term } => Action::Become(Role::Leader),
                    Message::AppendEntries { entries, term } => Action::Append(entries),
                    _ => todo!(),
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                Action::NOP
            }
        };
        Ok(action)
    }

    async fn listener_read_message(
        mut stream: TcpStream,
        role: Role,
        term: usize,
    ) -> tokio::io::Result<(TcpStream, Message)> {
        let mut buffer = [0; 1024];
        let message = match stream.read(&mut buffer).await {
            Ok(bytes_read) => {
                let message = bincode::deserialize::<Message>(buffer.as_slice()).unwrap();
                println!("<<< {:?} [{}]", message, stream.peer_addr().unwrap());

                match role {
                    Role::Follower => match message {
                        Message::RequestVote { candidate_id, term } => {
                            Message::Vote { candidate_id, term }
                        }
                        Message::RequestEntries { candidate_id, term } => todo!(),
                        _ => todo!(),
                    },
                    Role::Candidate => match message {
                        Message::RequestVote { candidate_id, term } => {
                            Message::Vote { candidate_id, term }
                        }
                        Message::RequestEntries { candidate_id, term } => Message::AppendEntries {
                            entries: vec![],
                            term,
                        },
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                todo!()
            }
        };

        Ok((stream, message))
    }

    async fn listener_write_message(
        mut stream: TcpStream,
        message: Message,
    ) -> tokio::io::Result<TcpStream> {
        stream
            .write_all(bincode::serialize(&message).unwrap().as_slice())
            .await
            .unwrap();
        stream.flush().await.unwrap();
        println!(">>> {:?} [{}]", message, stream.peer_addr().unwrap());

        Ok(stream)
    }
}
