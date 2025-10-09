use std::{error::Error, num::ParseIntError, sync::{atomic::AtomicU64, Arc, Weak}};

pub(crate) type VatId = Hash;
pub(crate) type ImportId = Hash;
pub(crate) type ExportId = ImportId;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{node::Kad, util::Hash};

pub(crate) struct ExportEntry {
    pub(crate) remote: bool,
    pub(crate) ref_count: AtomicU64,
}

pub(crate) struct ImportEntry {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Descriptor {
    ImportObject(u64),
    Export(u64),
}

pub struct Vat {
    pub id: VatId,
    pub(crate) kad: Weak<Kad>,
    pub(crate) exports: RwLock<Vec<ExportEntry>>,
    pub(crate) imports: RwLock<Vec<ImportEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RemoteRef {
    pub(crate) peer: Hash,
    pub(crate) swiss_num: String,
}

impl Vat {
    pub(crate) fn new(id: VatId, kad: Weak<Kad>) -> Self {
        Self {
            id,
            kad,
            imports: RwLock::new(Vec::new()),
            exports: RwLock::new(Vec::new()),
        }
    }
}

pub(crate) fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub(crate) fn b58es(s: &[u8]) -> String {
    bs58::encode(s).into_string()
}

pub(crate) fn b58d(s: &str) -> Option<Vec<u8>> {
    bs58::decode(s)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()
        .ok()
}

pub(crate) fn b58ds(h: Hash) -> Vec<u8> {
    let mut temp: Vec<u8> = Vec::with_capacity(32);
    h.to_little_endian(&mut temp[..]);

    bs58::encode(temp.as_slice())
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()
}
