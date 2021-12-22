use std::{collections::HashMap, str::FromStr};

use bitcoin::Txid;
use serde::de::{Deserialize, Deserializer};
use vrsc::Address;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identity {
    pub identity: InnerIdentity,
    pub status: String,
    pub canspendfor: bool,
    pub cansignfor: bool,
    pub blockheight: i64,
    pub txid: String, // TODO hash
    pub vout: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InnerIdentity {
    pub version: u16,
    pub flags: u16,
    pub primaryaddresses: Vec<Address>,
    pub minimumsignatures: u16,
    pub name: String,
    pub identityaddress: Address,
    pub parent: Address,
    pub systemid: Address,
    pub contentmap: HashMap<String, String>,
    pub revocationauthority: Address,
    pub recoveryauthority: Address,
    pub privateaddress: String,
    pub timelock: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameCommitment {
    txid: Txid,
    namereservation: NameReservation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameReservation {
    name: String,
    salt: String,
    // if no refferal was given, the response is an empty string.
    #[serde(deserialize_with = "object_empty_as_none")]
    referral: Option<Address>,
    parent: String,
    nameid: Address,
}

pub fn object_empty_as_none<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    } else {
        return Ok(Some(Address::from_str(&s).unwrap()));
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketplaceOffer {
    identityid: Address,
    price: f64,
    offer: Offer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Offer {
    accept: OfferVariant,
    offer: OfferVariant,
    blockexpiry: u64,
    txid: Txid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OfferVariant {
    CurrencyOffer(HashMap<String, f64>),
    IdentityOffer(IdentityOffer),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityOffer {
    name: String,
    identityid: Address,
    systemid: Address,
    original: u8,
}
