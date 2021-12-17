use std::collections::HashMap;

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
