#![allow(unused)]

use std::{
    net::{SocketAddr, SocketAddrV4},
    time::Duration,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    Leader,
    Follower,
    Candidate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    RequestVote {
        candidate_id: SocketAddr,
        term: usize,
    },
    Vote {
        candidate_id: SocketAddr,
        term: usize,
    },
    RequestEntries {
        candidate_id: SocketAddr,
        term: usize,
    },
    AppendEntries {
        entries: Vec<String>,
        term: usize,
    },
}

#[derive(Debug)]
pub struct Node {
    pub addr: SocketAddrV4,
    // pub term: usize,
    pub nodes: Vec<SocketAddrV4>,
    pub delay: Duration,
    pub role: Role,
    pub term: usize,
    pub votes: usize,
    // pub voted_for: Option<usize>,
    pub log: Vec<String>,
}

impl Node {
    pub fn new(id: usize, addr: SocketAddrV4, nodes: Vec<SocketAddrV4>) -> Self {
        let rnd = rand::thread_rng().gen_range(3..5);
        Self {
            addr,
            term: 0,
            nodes,
            // connections: vec![],
            votes: 0,
            delay: Duration::from_secs(rnd),
            role: Role::Candidate,
            // voted_for: None,
            log: vec![],
        }
    }
}
