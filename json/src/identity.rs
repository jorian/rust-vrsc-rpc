use std::{collections::HashMap, str::FromStr};

use bitcoin::{
    hashes::{ripemd160::Hash as hash160, sha256::Hash as hash256},
    Txid,
};
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
    pub privateaddress: Option<String>,
    pub timelock: u64,
    pub txout: Option<InnerIdentityTxOut>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InnerIdentityTxOut {
    pub txid: Txid,
    pub voutnum: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameCommitment {
    pub txid: Txid,
    pub namereservation: NameReservation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameReservation {
    pub name: String,
    pub salt: String,
    pub version: u8,
    // if no refferal was given, the response is an empty string.
    #[serde(deserialize_with = "object_empty_as_none")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral: Option<Address>,
    pub parent: String,
    pub nameid: Address,
}

pub fn object_empty_as_none<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    } else {
        return Ok(Some(
            Address::from_str(&s).expect("a valid Verus i, b, or R address"),
        ));
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketplaceOffer {
    pub identityid: Address,
    pub price: f64,
    pub offer: Offer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Offer {
    pub accept: OfferVariant,
    pub offer: OfferVariant,
    pub blockexpiry: u64,
    pub txid: Txid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OfferVariant {
    CurrencyOffer(HashMap<String, f64>),
    IdentityOffer(IdentityOffer),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityOffer {
    pub name: String,
    pub identityid: Address,
    pub systemid: Address,
    pub original: u8,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
pub type IdentitiesWithAddressResult = Vec<InnerIdentity>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetVDXFIdResult {
    pub vdxfid: Address,
    pub hash160result: hash160,
    pub qualifiedname: Option<QualifiedName>,
    pub bounddata: Option<BoundData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QualifiedName {
    pub name: String,
    pub parentid: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoundData {
    pub vdxfkey: Address,
    pub uint256: hash256,
    pub indexnum: u32,
}
