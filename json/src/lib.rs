#![crate_name = "komodo_rpc_json"]
#![crate_type = "rlib"]

pub extern crate bitcoin;
pub extern crate komodo;

#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

use crate::komodo::SignedAmount;
use bitcoin::{BlockHash, PubkeyHash, Script, ScriptHash, Txid};
use komodo::util::amount::Amount;
pub use komodo::Address;
use komodo::{PrivateKey, PublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use bitcoin::hash_types::*;

#[derive(Clone, Debug)]
pub enum PubkeyOrAddress<'a> {
    Address(&'a Address),
    Pubkey(&'a str),
}

impl<'a> serde::Serialize for PubkeyOrAddress<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            PubkeyOrAddress::Address(a) => serde::Serialize::serialize(a, serializer),
            PubkeyOrAddress::Pubkey(p) => serde::Serialize::serialize(p, serializer),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoinSupply {
    pub result: String,
    pub coin: String,
    pub height: i32,
    pub supply: f64,
    #[serde(rename = "zfunds")]
    pub z_funds: f64,
    pub sprout: f64,
    pub total: f64,
    #[serde(rename = "lastmonth")]
    pub last_month: Option<f64>,
    #[serde(rename = "monthcoins")]
    pub month_coins: Option<f64>,
    #[serde(rename = "lastquarter")]
    pub last_quarter: Option<f64>,
    #[serde(rename = "quartercoins")]
    pub quarter_coins: Option<f64>,
    #[serde(rename = "lastyear")]
    pub last_year: Option<f64>,
    #[serde(rename = "yearcoins")]
    pub year_coins: Option<f64>,
    pub inflation: Option<f64>,
    #[serde(rename = "blocksperyear")]
    pub blocks_per_year: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub last_notarized_height: u32,
    pub hash: bitcoin::BlockHash,
    pub confirmations: u32,
    #[serde(rename = "rawconfirmations")]
    pub raw_confirmations: u32,
    pub size: u32,
    pub height: u32,
    pub version: u16,
    #[serde(rename = "merkleroot")]
    pub merkle_root: bitcoin::TxMerkleNode,
    #[serde(rename = "segid")]
    pub seg_id: i32,
    #[serde(rename = "finalsaplingroot")]
    pub final_sapling_root: String,
    pub tx: Vec<bitcoin::hash_types::Txid>,
    pub time: u64,
    pub nonce: String,
    pub solution: String,
    pub bits: String,
    pub difficulty: f64,
    #[serde(rename = "chainwork")]
    pub chain_work: String,
    pub anchor: String,
    #[serde(rename = "blocktype")]
    pub block_type: String,
    #[serde(rename = "valuePools")]
    pub value_pools: Vec<ValuePool>,
    #[serde(rename = "previousblockhash")]
    pub previous_blockhash: Option<bitcoin::BlockHash>,
    #[serde(rename = "nextblockhash")]
    pub next_blockhash: Option<bitcoin::BlockHash>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValuePool {
    pub id: String,
    pub monitored: bool,
    #[serde(rename = "chainValue")]
    pub chain_value: f64,
    #[serde(rename = "chainValueZat")]
    pub chain_value_sat: u64,
    #[serde(rename = "valueDelta")]
    pub value_delta: f64,
    #[serde(rename = "valueDeltaZat")]
    pub value_delta_sat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletInfo {
    #[serde(rename = "walletversion")]
    pub wallet_version: u32,
    pub balance: f64,
    pub unconfirmed_balance: f64,
    pub immature_balance: f64,
    #[serde(rename = "txcount")]
    pub tx_count: u32,
    #[serde(rename = "keypoololdest")]
    pub keypool_oldest: u64,
    #[serde(rename = "keypoolsize")]
    pub keypool_size: u32,
    pub unlocked_until: Option<u32>,
    #[serde(rename = "paytxfee")]
    pub pay_tx_fee: f64,
    // Todo what is this?
    #[serde(rename = "seedfp")]
    pub seed_fp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CleanedWalletTransactions {
    #[serde(rename = "total_transactons")]
    pub total: u8,
    #[serde(rename = "remaining_transactons")]
    pub remaining: u8,
    #[serde(rename = "removed_transactions")]
    pub removed: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConvertedPassphrase {
    #[serde(rename = "agamapassphrase")]
    pub passphrase: String,
    pub address: Address,
    #[serde(rename = "pubkey")]
    pub public_key: PublicKey,
    #[serde(rename = "privkey")]
    pub private_key: PrivateKey,
    pub wif: PrivateKey,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransactionResult {
    pub amount: f64,
    pub fee: Option<f64>,
    pub rawconfirmations: u32,
    pub confirmations: u32,
    pub blockhash: Option<bitcoin::BlockHash>,
    pub blockindex: u32,
    pub blocktime: Option<u64>,
    pub expiryheight: u32,
    pub txid: bitcoin::Txid,
    pub walletconflicts: Vec<Option<bitcoin::Txid>>,
    pub time: u64,
    pub timereceived: u64,
    pub vjoinsplit: Vec<Option<GetTransactionVJoinSplit>>,
    pub hex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransactionVJoinSplit {
    pub anchor: String, // Merkle root
    pub nullifiers: Vec<Option<String>>,
    pub commitments: Vec<Option<String>>,
    pub macs: Vec<Option<String>>,
    pub vpub_old: f64,
    pub vpub_new: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransactionDetails {
    account: String,
    pub address: Address,
    pub category: GetTransactionDetailsCategory,
    pub amount: f64,
    pub vout: u16,
    pub fee: Option<f64>,
    pub size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GetTransactionDetailsCategory {
    Send,
    Receive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListLockUnspentResult {
    pub txid: bitcoin::Txid,
    pub vout: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListReceivedByAddressResult {
    #[serde(rename = "involvesWatchonly")]
    pub involves_watch_only: Option<bool>,
    pub address: Address,
    account: String,
    #[serde(with = "komodo::util::amount::serde::as_kmd")]
    pub amount: Amount,
    pub confirmations: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListSinceBlockResult {
    pub transactions: Vec<ListSinceBlockTransactions>,
    pub lastblock: BlockHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListSinceBlockTransactions {
    account: String,
    pub address: Option<Address>,
    pub category: ListSinceBlockCategory,
    #[serde(with = "komodo::util::amount::serde::as_kmd")]
    pub amount: SignedAmount,
    pub vout: u16,
    #[serde(with = "komodo::util::amount::serde::as_kmd::opt", default)]
    pub fee: Option<SignedAmount>,
    pub confirmations: u32,
    pub blockhash: BlockHash,
    pub blockindex: u32,
    pub blocktime: u64,
    pub txid: Txid,
    pub time: u64,
    pub timereceived: u64,
    pub comment: Option<String>,
    pub to: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ListSinceBlockCategory {
    #[serde(rename = "send")]
    Send,
    #[serde(rename = "receive")]
    Receive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListTransactionsResult {
    account: String,
    pub address: Address,
    pub category: ListSinceBlockCategory,
    #[serde(with = "komodo::util::amount::serde::as_kmd")]
    pub amount: SignedAmount,
    pub vout: u16,
    #[serde(with = "komodo::util::amount::serde::as_kmd::opt", default)]
    pub fee: Option<SignedAmount>,
    pub confirmations: u32,
    pub blockhash: BlockHash,
    pub blockindex: u32,
    pub txid: Txid,
    pub time: u64,
    pub timereceived: u64,
    pub comment: Option<String>,
    otheraccount: Option<String>,
    pub size: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListUnspentResult {
    pub txid: Txid,
    pub vout: u16,
    pub generated: bool,
    pub address: Option<Address>,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Script,
    #[serde(with = "komodo::util::amount::serde::as_kmd", default)]
    pub amount: SignedAmount,
    pub confirmations: u32,
    #[serde(rename = "redeemScript")]
    pub redeem_script: Option<Script>,
    pub spendable: bool,
}
