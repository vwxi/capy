mod crypto;
pub mod forward;
mod lookup;
pub mod node;
mod routing;
mod rpc;
mod score;
mod store;
pub mod util;
pub mod vat;

uint::construct_uint!(
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct U256(4);
);
