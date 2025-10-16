use std::{collections::HashMap, error::Error, num::ParseIntError, sync::{atomic::AtomicU64, Arc, Weak}};

pub(crate) type VatId = Hash;
pub(crate) type ImportId = Hash;
pub(crate) type ExportId = ImportId;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{node::Kad, util::Hash};

mod consts {
    pub(super) const FETCH_SWISS: &'static str = "fetch";
    pub(super) const DEPOSIT_GIFT_SWISS: &'static str = "deposit_gift";
    pub(super) const WITHDRAW_GIFT_SWISS: &'static str = "withdraw_gift";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    None,
    Err(String),
    Bool(bool),
    Integer(i64),
    // ugh. ugh. ugh!!!
    Float(String),
    ByteArray(Vec<u8>),
}

pub type Method = fn(Vec<Value>) -> Value;

pub struct Object {
    pub(crate) fields: HashMap<String, Value>,
    pub(crate) methods: HashMap<String, Method>,
}

pub(crate) struct ExportEntry {
    pub(crate) object: Option<Object>,
    pub(crate) ref_count: AtomicU64,
}

pub(crate) struct ImportEntry {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum Descriptor {
    SenderHosted(ImportId),
    SenderPromise(ImportId),
    ReceiverHosted(ExportId),
    // TODO: answer
    #[allow(unused)]
    ReceiverAnswer(),
}

pub struct Vat {
    pub id: VatId,
    pub(crate) kad: Weak<Kad>,
    pub(crate) sessions: RwLock<HashMap<VatId, Arc<Session>>>,
}

pub struct Session {
    pub(crate) vat: Weak<Vat>,
    pub(crate) exports: RwLock<HashMap<ExportId, ExportEntry>>,
    pub(crate) imports: RwLock<HashMap<ImportId, ImportEntry>>,
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
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Session {
    pub(crate) fn new(vat: Weak<Vat>) -> Self {
        Self {
            vat,
            exports: RwLock::new(HashMap::new()),
            imports: RwLock::new(HashMap::new()),
        }
    }

    fn fetch(args: Vec<Value>) -> Value {
        Value::None
    }

    fn deposit_gift(args: Vec<Value>) -> Value {
        Value::None
    }

    fn withdraw_gift(args: Vec<Value>) -> Value {
        Value::None
    }

    pub(crate) fn make_bootstrap(&self) -> Object {
        Object {
            fields: HashMap::new(),
            methods: HashMap::from([
                (String::from(consts::FETCH_SWISS), Self::fetch as Method),
                (String::from(consts::DEPOSIT_GIFT_SWISS), Self::deposit_gift as Method),
                (String::from(consts::WITHDRAW_GIFT_SWISS), Self::withdraw_gift as Method),
            ])
        }
    }

    pub(crate) async fn export(&self, id: ExportId, entry: ExportEntry) -> bool {
        let mut exports_lock = self.exports.write().await;

        exports_lock.insert(id, entry).is_none()
    }

    pub(crate) async fn import(&self, id: ImportId, entry: ImportEntry) -> bool {
        let mut imports_lock = self.imports.write().await;

        imports_lock.insert(id, entry).is_none()
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
