//! Utility functions
//!
//! Structures, type aliases and helper functions

use core::fmt;
use std::{
    net::IpAddr,
    num::ParseIntError,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use rand::{distributions::Standard, prelude::Distribution, Rng};
use rsa::sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Addr(pub IpAddr, pub u16);
pub type Hash = crate::U256;
pub type Kvs<T> = Vec<Kv<T>>;

macro_rules! pred_block {
    ($( #[$meta:meta] {$($item:item)*} )*) => {
        $($(
            #[$meta]
            $item
        )*)*
    }
}

pub(crate) use pred_block;

impl Addr {
    pub(crate) fn to(&self) -> (IpAddr, u16) {
        (self.0, self.1)
    }
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

use crate::{store::StoreEntry, vat::RemoteRef};

// a peer object with multiple addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: Hash,
    pub addresses: Vec<(Addr, usize)>,
}

impl std::hash::Hash for Peer {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        let mut temp: Vec<u8> = Vec::with_capacity(32);
        self.id.to_little_endian(&mut temp[..]);
        state.write(&temp[..]);
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.addresses.iter().any(|x| other.addresses.contains(x))
    }
}

impl Eq for Peer {}

impl Peer {
    #[must_use]
    pub fn new(id_: Hash, addr: Addr) -> Self {
        Peer {
            id: id_,
            addresses: vec![(addr, 0)],
        }
    }

    /// Return single peer from peer
    ///
    /// # Errors
    ///
    /// Returns an `Err` if there are no addresses in the address list.
    pub fn single_peer(&self) -> Result<SinglePeer> {
        Ok(SinglePeer {
            id: self.id,
            addr: self
                .addresses
                .first()
                .ok_or(anyhow!("no addresses in peer object"))?
                .0,
        })
    }
}

// a peer object with a single address
#[derive(Debug, Copy, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SinglePeer {
    pub id: Hash,
    pub addr: Addr,
}

impl SinglePeer {
    #[must_use]
    pub fn new(id_: Hash, addr_: Addr) -> Self {
        SinglePeer {
            id: id_,
            addr: addr_,
        }
    }

    #[must_use]
    pub fn peer(&self) -> Peer {
        Peer {
            id: self.id,
            addresses: vec![(self.addr, 0)],
        }
    }
}

impl Distribution<Hash> for Standard {
    fn sample<R: Rng + ?Sized>(&self, _: &mut R) -> Hash {
        let i = (0..32u8).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
        Hash::from(&i[..])
    }
}

/// Hashes a string slice.
#[must_use]
pub fn hash(s: &str) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());

    Hash::from_little_endian(hasher.finalize().as_mut_slice())
}

pub fn b58d(s: &str) -> Option<Hash> {
    Some(Hash::from_little_endian(
        bs58::decode(s)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
            .ok()?
            .as_mut_slice(),
    ))
}

pub fn b58e(h: Hash) -> String {
    let mut temp: Vec<u8> = vec![0; 32];
    h.to_little_endian(&mut temp[..]);

    bs58::encode(temp.as_slice()).into_string()
}

/// shorten hash to string
pub fn shh(h: Hash) -> String {
    let s = b58e(h);
    let end = s.len() - 5;
    let mut st = String::new();

    st += &s[0..5];
    st += "...";
    st += &s[end..];

    st
}

pub(crate) fn timestamp() -> u64 {
    let t = SystemTime::now();
    t.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub(crate) fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[cfg(test)]
pub(crate) fn generate_peer(pid: Option<Hash>) -> SinglePeer {
    use std::net::Ipv4Addr;

    SinglePeer {
        id: if let Some(pid_) = pid {
            pid_
        } else {
            rand::random()
        },
        addr: Addr(IpAddr::V4(Ipv4Addr::LOCALHOST), rand::random()),
    }
}

crate::util::pred_block! {
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)] {
        pub struct ProviderRecord {
            pub provider: Hash,
            pub expiry: u64,
        }

        pub enum Data {
            Raw(Vec<u8>),
            Compressed(Vec<u8>)
        }

        pub enum Value {
            Data(Data),
            ProviderRecord(ProviderRecord),
        }

        pub struct Kv<T> {
            pub value: T,
            pub origin: SinglePeer,
            pub timestamp: u64
        }

        pub(crate) struct Entry {
            pub value: Value,
            pub signature: String,
            pub origin: SinglePeer,
            pub timestamp: u64,
        }

        pub(crate) enum RpcOp {
            // maintenance
            Nothing,
            Key,
            Ping,
            GetAddresses(Hash),
            GetConfidence(Hash),

            // kv
            FindNode(Hash),
            FindValue(Hash),
            Store(Hash, Box<StoreEntry>),

            // objects
            StartSession,
            Abort(String),
            DeliverOnly(crate::vat::Descriptor, String, Vec<crate::vat::Value>),
        }

        pub(crate) enum FindValueResult {
            None,
            Value(Box<StoreEntry>),
            Nodes(Vec<SinglePeer>),
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)] {
        pub(crate) enum RpcResult {
            Bad(String),
            Key(String),
            Ping,
            Store,
            GetAddresses(Option<Vec<Addr>>),
            GetConfidence(f64),
            FindNode(Vec<SinglePeer>),
            FindValue(Box<FindValueResult>),
            StartSession(RemoteRef),
            Abort,
            DeliverOnly,
        }

        pub(crate) struct RpcContext {
            pub(crate) version: String,
            pub(crate) id: Hash,
            pub(crate) op: RpcOp,
            pub(crate) addr: Addr,
            pub(crate) timestamp: u64,
        }
    }
}

pub(crate) type RpcArgs = (RpcContext, String);
pub(crate) type RpcResults = (RpcResult, RpcContext, String);
