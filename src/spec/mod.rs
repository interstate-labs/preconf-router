use alloy::{hex, primitives::{keccak256, Address, Signature, B256}};
use reth_primitives::PooledTransactionsElement;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposer {
    pub source: String,
    pub slot: u64,
    pub sidecar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreconfRequestParams {
    pub proposer: Proposer,
    pub signer: Address,
    #[serde(deserialize_with = "deserialize_sig", serialize_with = "serialize_sig")]
    pub sidecar_signature: Signature,
    #[serde(deserialize_with = "deserialize_sig", serialize_with = "serialize_sig")]
    pub signature: Signature,
    
    #[serde(deserialize_with = "deserialize_tx", serialize_with = "serialize_tx")]
    pub signed_tx: PooledTransactionsElement,
}

impl PreconfRequestParams {
    pub fn digest(&self) -> B256 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.proposer.slot.to_le_bytes());
        data.extend_from_slice(self.signed_tx.hash().as_slice());

        keccak256(&data)
    }
}


#[derive(Debug, Deserialize)]
pub struct BoltSidecar {
    pub slot: u64,
    pub validator_index: u32,
    pub validator_pubkey: String,
    pub sidecar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct InterstateSidecar {
    pub validator_index: u32,
    pub sidecar_url: String,
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
pub struct PrimevItems {
    pub items: std::collections::HashMap<String, PrimevItem>,
}

#[derive(Debug, Deserialize)]
pub struct PrimevItem {
    pub isOptedIn: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sidecar {
    pub validator_index: u32,
    pub sidecar_url: String,
    pub source: String,
    pub slot: u64,
}

fn deserialize_tx<'de, D>(deserializer: D) -> Result<PooledTransactionsElement, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let data = hex::decode(s.trim_start_matches("0x")).map_err(de::Error::custom)?;
    tracing::debug!("in deserialization {}", s);
    PooledTransactionsElement::decode_enveloped(&mut data.as_slice()).map_err(de::Error::custom)
}

pub(crate) fn serialize_tx<S>(
    tx: &PooledTransactionsElement,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut data = Vec::new();
    tx.encode_enveloped(&mut data);
    tracing::debug!("in serialization 0x{:#?}", data);
    serializer.serialize_str(&format!("0x{}", hex::encode(&data)))
}


fn deserialize_sig<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(s.trim_start_matches("0x")).map_err(de::Error::custom)
}

pub fn serialize_sig<S: serde::Serializer>(sig: &Signature, serializer: S) -> Result<S::Ok, S::Error> {
    let parity = sig.v();
    // As bytes encodes the parity as 27/28, need to change that.
    let mut bytes = sig.as_bytes();
    bytes[bytes.len() - 1] = if parity.y_parity() { 1 } else { 0 };
    serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}
