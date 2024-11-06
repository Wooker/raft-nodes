#![allow(unused)]

use std::{
    collections::HashMap,
    fmt::Display,
    net::{SocketAddr, SocketAddrV4},
    time::Duration,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::flow_entry::{FlowModEntries, SwitchData};

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
        entries: HashMap<SocketAddr, SwitchData>,
        term: usize,
    },
    AppendEntries {
        entries: HashMap<SocketAddr, SwitchData>,
        term: usize,
    },
}

#[derive(Debug)]
pub struct Node {
    pub addr: SocketAddrV4,
    // pub term: usize,
    pub controller_port: usize,
    pub nodes: Vec<SocketAddrV4>,
    pub delay: Duration,
    pub role: Role,
    pub term: usize,
    pub votes: usize,
    // pub voted_for: Option<usize>,
    pub log: HashMap<SocketAddr, SwitchData>,
}

impl Node {
    pub fn new(
        id: usize,
        addr: SocketAddrV4,
        controller_port: usize,
        nodes: Vec<SocketAddrV4>,
    ) -> Self {
        let rnd = rand::thread_rng().gen_range(3..5);

        let mut map = HashMap::new();
        for n in nodes.iter() {
            map.insert(
                SocketAddr::V4(n.clone()),
                HashMap::<String, Vec<FlowModEntries>>::new(),
            );
        }
        Self {
            addr,
            term: 0,
            nodes,
            controller_port,
            // connections: vec![],
            votes: 0,
            delay: Duration::from_secs(rnd),
            role: Role::Candidate,
            // voted_for: None,
            log: map,
        }
    }
}
