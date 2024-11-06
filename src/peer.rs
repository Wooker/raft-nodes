#![allow(unused)]
#![allow(dead_code)]

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

impl Peer {
    pub fn new(node: Node, delay: Duration) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
            delay,
        }
    }

    // Connection handler for accepted connections
    async fn handle_connection(addr: SocketAddr, mut stream: TcpStream) -> std::io::Result<()> {
        use tokio::io::AsyncReadExt;

        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(bytes_read) => {
                let str = "Hello";
                println!("<<< {} [{}]", bytes_read, addr);
                stream.write_all(str.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
                println!(">>> {}", str);
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> io::Result<()> {
        println!("Starting server...");

        loop {
            // Step 1: Run listener for a specific duration
            self.run_listener().await?;

            // Step 2: After listener duration, switch to client mode
            println!("Listener duration expired. Switching to client mode.");
            self.run_sender().await?;

            println!("Finished client connections. Returning.");
        }
        Ok(())
    }

    async fn run_listener(&mut self) -> io::Result<()> {
        let addr = { self.node.read().unwrap().addr };
        let listen_duration = self.delay;

        'listener: loop {
            // Create a new TcpSocket for listening
            let sock = TcpSocket::new_v4().unwrap();
            sock.set_reuseaddr(true).unwrap();
            sock.bind(std::net::SocketAddr::V4(addr))
                .expect("Could not bind the socket for listening");

            println!("Listener started on {:?}", addr);
            let listener = sock.listen(1024).unwrap();

            let accept_timeout = self.delay;
            match time::timeout(accept_timeout, listener.accept()).await {
                Ok(Ok((stream, addr))) => {
                    Self::handle_connection(addr, stream).await.unwrap();
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
        let (addr, nodes) = {
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
            )
        };

        'sender: loop {
            for &node_addr in &nodes {
                // Create a new TcpSocket for each connection attempt
                let sock = TcpSocket::new_v4().unwrap();
                sock.set_reuseaddr(true).unwrap();
                sock.bind(SocketAddr::V4(addr))
                    .expect("Could not bind the socket for stream");

                match sock.connect(SocketAddr::V4(node_addr)).await {
                    Ok(mut stream) => {
                        let message = "Hello from client!";
                        stream.write_all(message.as_bytes()).await.unwrap();
                        stream.flush().await.unwrap();
                        println!(">>> {} [{}]", message, node_addr);

                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer).await {
                            Ok(bytes_read) => {
                                let str = "Hello";
                                println!("<<< {} [{}]", bytes_read, addr);
                                stream.write_all(str.as_bytes()).await.unwrap();
                                stream.flush().await.unwrap();
                                println!(">>> {}", str);
                            }
                            Err(e) => {
                                eprintln!("Failed to read from stream: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect as a client: {}", e);
                    }
                }
            }

            println!();
            thread::sleep(self.delay - Duration::from_secs(1));
        }

        Ok(())
    }

    async fn handle_leader(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        println!("Leader handling incoming connection");
        // Leader-specific logic here
    }

    async fn handle_follower(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        // Follower-specific logic here
        let mut buf = String::new();
        stream.read_to_string(&mut buf).await.unwrap();
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
            .await
            .unwrap();
        stream.flush().await.unwrap();
    }

    async fn handle_candidate(node: &Arc<RwLock<Node>>, stream: &mut TcpStream) {
        println!("Candidate handling incoming connection");

        // Read data outside of lock
        let mut buf = String::new();
        stream.read_to_string(&mut buf).await.unwrap();

        // Only acquire write lock when needed
        let mut node_write = node.write().unwrap();
        node_write.role = Role::Follower;
        println!("Now Follower");

        // Flush and finalize handling
        stream.flush().await.unwrap();
    }

    async fn leader_periodic_task(node: &Arc<RwLock<Node>>) {
        println!("Leader periodic task");
        // Implement leader-specific periodic task
    }

    async fn follower_periodic_task(node: &Arc<RwLock<Node>>, delay: Duration) {
        println!("Follower periodic task");

        // Implement follower-specific task loop
        loop {
            sleep(delay);
        }
    }

    async fn candidate_periodic_task(node: &Arc<RwLock<Node>>) {
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

        /*
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
        */
    }
}
