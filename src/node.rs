#![allow(unused)]

use std::{net::SocketAddrV4, time::Duration};

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
        candidate_id: usize,
        // candidate_term: usize,
    },
    Vote {
        candidate_id: usize,
    },
    RequestEntries {
        candidate_id: usize,
    },
    AppendEntries {
        entries: Vec<LogEntry>,
        leader_term: usize,
    },
}

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub addr: SocketAddrV4,
    // pub term: usize,
    pub nodes: Vec<SocketAddrV4>,
    pub delay: Duration,
    pub role: Role,
    // pub voted_for: Option<usize>,
    // log: Vec<LogEntry>,
}

impl Node {
    pub fn new(id: usize, addr: SocketAddrV4, nodes: Vec<SocketAddrV4>) -> Self {
        let rnd = rand::thread_rng().gen_range(3..5);
        Self {
            id,
            addr,
            // term: 0,
            nodes: nodes,
            // connections: vec![],
            delay: Duration::from_secs(rnd),
            role: Role::Candidate,
            // voted_for: None,
            // log: vec![],
        }
    }
}
