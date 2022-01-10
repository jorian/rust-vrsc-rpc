#![crate_name = "vrsc_rpc_json"]
#![crate_type = "rlib"]

#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

pub extern crate bitcoin;
pub extern crate vrsc;

pub mod identity;

use crate::vrsc::{Address, Amount, PrivateKey, PublicKey, SignedAmount};

use bitcoin::{BlockHash, Script, Txid};
use serde::*;
use std::{collections::HashMap, fmt::Display, num::ParseIntError, str, str::FromStr};

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
pub struct MiningInfo {
    pub blocks: u64,
    pub currentblocksize: u32,
    pub currentblocktx: u16,
    pub averageblockfees: f64,
    pub difficulty: f64,
    pub stakingsupply: f64,
    pub errors: String,
    pub genproclimit: u16,
    pub localhashps: f64,
    pub networkhashps: f64,
    pub pooledtx: u16,
    pub testnet: bool,
    pub chain: String,
    pub generate: bool,
    pub staking: bool,
    pub numthreads: u16,
    pub mergemining: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddressUtxos {
    pub addresses: Option<Vec<Address>>,
    pub address: Address,
    pub txid: Txid,
    pub outputIndex: u16,
    pub script: String,
    pub satoshis: u64,
    pub height: u64,
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
    // pub last_notarized_height: u32,
    pub hash: bitcoin::BlockHash,
    #[serde(rename = "validationtype")]
    pub validation_type: ValidationType, //todo make this an Enum
    pub postarget: Option<String>,
    pub poshashbh: Option<String>,
    pub poshashtx: Option<String>,
    pub possourcetxid: Option<Txid>,
    pub possourcevoutnum: Option<u32>,
    pub posrewarddest: Option<Address>,
    pub postxddest: Option<Address>,
    pub confirmations: u32,
    // #[serde(rename = "rawconfirmations")]
    // pub raw_confirmations: u32,
    pub size: u32,
    pub height: u32,
    pub version: u32,
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
    #[serde(rename = "chainstake")]
    pub chain_stake: String,
    pub anchor: String,
    #[serde(rename = "blocktype")]
    pub block_type: String,
    #[serde(rename = "valuePools")]
    pub value_pools: Vec<ValuePool>,
    #[serde(rename = "previousblockhash")]
    pub previous_blockhash: Option<bitcoin::BlockHash>,
    #[serde(rename = "nextblockhash")]
    pub next_blockhash: Option<bitcoin::BlockHash>,
    pub proofroot: ProofRoot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ValidationType {
    #[serde(rename = "stake")]
    Stake,
    #[serde(rename = "work")]
    Work,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProofRoot {
    pub version: u32,
    #[serde(rename = "type")]
    pub proof_type: u32,
    pub systemid: Address,
    pub height: u64,
    pub stateroot: String,
    pub blockhash: BlockHash,
    pub power: String,
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
    pub value_delta: Option<f64>,
    #[serde(rename = "valueDeltaZat")]
    pub value_delta_sat: Option<i64>,
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
    pub blockindex: Option<u32>,
    pub blocktime: Option<u64>,
    pub expiryheight: Option<u32>,
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
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
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
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub amount: SignedAmount,
    pub vout: u16,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc::opt", default)]
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
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub amount: SignedAmount,
    pub vout: u16,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc::opt", default)]
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
    #[serde(with = "vrsc::util::amount::serde::as_vrsc", default)]
    pub amount: SignedAmount,
    pub confirmations: u32,
    #[serde(rename = "redeemScript")]
    pub redeem_script: Option<Script>,
    pub spendable: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionResultVerbose {
    pub hex: String,
    pub txid: bitcoin::Txid,
    pub version: u32,
    pub locktime: u64,
    pub expiryheight: u32,
    pub vin: Vec<GetRawTransactionVin>,
    pub vout: Vec<GetRawTransactionVout>,
    pub vjoinsplit: Vec<GetRawTransactionVJoinSplit>,
    pub blockhash: Option<bitcoin::BlockHash>, // transaction might not be in a block yet
    pub height: Option<u32>,
    pub confirmations: Option<u32>,
    pub rawconfirmations: Option<u32>,
    pub time: Option<u64>,
    pub blocktime: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionVin {
    pub txid: bitcoin::Txid,
    pub vout: u32,
    #[serde(rename = "scriptSig")]
    pub script_sig: GetRawTransactionVinScriptSig,
    pub sequence: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionVinScriptSig {
    pub asm: String,
    pub hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionVout {
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub value: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_sat")]
    pub valueSat: Amount,
    // #[serde(with = "vrsc::util::amount::serde::as_kmd::opt")]
    // pub interest: Option<Amount>,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: GetRawTransactionVoutScriptPubKey,
    pub spentTxId: Option<Txid>,
    pub spentIndex: Option<u32>,
    pub spentHeight: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionVoutScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub required_sigs: Option<u32>,
    #[serde(rename = "type")]
    pub _type: String,
    pub addresses: Option<Vec<Address>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionVJoinSplit {
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub vpub_old: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub vpub_new: Amount,
    pub anchor: String,
    // TODO hexes:
    pub nullifiers: Vec<String>,
    pub commitments: Vec<String>,
    #[serde(rename = "onetimePubKey")]
    pub onetime_pubkey: String,
    #[serde(rename = "randomSeed")]
    pub random_seed: String,
    pub macs: Vec<String>,
    pub proof: String,
    pub ciphertexts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionResult(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpReturnBurnResult {
    hex: String,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct SetPubkeyResult {
//     pub ismine: String,
//     pub address: Address,
//     pub pubkey: PublicKey,
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfo {
    // Can be one of main, test or regtest
    pub chain: String,
    pub name: String,
    pub chainid: Address,
    pub blocks: u32,
    // pub synced: bool,
    pub headers: u32,
    pub bestblockhash: bitcoin::BlockHash,
    pub difficulty: f64,
    pub verificationprogress: f64,
    pub chainwork: String,
    pub chainstake: String,
    pub pruned: bool,
    pub size_on_disk: u64,
    pub commitments: u64,
    #[serde(rename = "valuePools")]
    pub value_pools: Vec<ValuePool>,
    pub softforks: Vec<BlockchainInfoSoftfork>,
    pub upgrades: Option<HashMap<String, BlockchainInfoUpgrade>>,
    pub consensus: BlockchainInfoConsensus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfoConsensus {
    pub chaintip: String,
    pub nextblock: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfoUpgrade {
    pub name: String,
    pub activationheight: u32,
    pub status: String,
    pub info: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfoSoftfork {
    pub id: String,
    pub version: u32,
    pub enforce: BlockchainInfoSoftforkProgress,
    pub reject: BlockchainInfoSoftforkProgress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfoSoftforkProgress {
    pub status: bool,
    pub found: u32,
    pub required: u32,
    pub window: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockHeader {
    pub hash: bitcoin::BlockHash,
    pub confirmations: u32,
    pub height: u32,
    pub version: u32,
    pub merkleroot: String,
    pub time: u32,
    pub nonce: String,
    pub solution: String,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub segid: i32,
    pub previousblockhash: Option<bitcoin::BlockHash>, // oldest block has no previous block
    pub nextblockhash: Option<bitcoin::BlockHash>,     // newest block has no next block
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainTips(Vec<ChainTip>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainTip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u32,
    pub status: ChainTipStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainTipStatus {
    Invalid,
    #[serde(rename = "headers-only")]
    HeadersOnly,
    #[serde(rename = "valid-headers")]
    ValidHeaders,
    #[serde(rename = "valid-fork")]
    ValidFork,
    Active,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainTxStats {
    pub time: u64,
    pub txcount: u64,
    pub window_final_block_hash: bitcoin::BlockHash,
    pub window_block_count: u32,
    pub window_tx_count: u64,
    pub window_interval: u64,
    pub txrate: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolInfo {
    pub size: u32,
    pub bytes: u32,
    pub usage: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawMempool(HashMap<String, RawMempoolTransactionInfo>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawMempoolTransactionInfo {
    pub size: u32,
    pub fee: f32,
    pub time: u32,
    pub height: u32,
    pub startingpriority: f64,
    pub currentpriority: f64,
    pub depends: Vec<String>, // this either returns an empty array or an array with txids
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpentInfoResult {
    pub txid: bitcoin::Txid,
    pub index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxOutResult {
    pub bestblock: BlockHash,
    pub confirmations: u32,
    pub rawconfirmations: u32,
    pub value: f64,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: ScriptPubKey,
    pub version: u32,
    pub coinbase: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: u32,
    #[serde(rename = "type")]
    pub script_type: String,
    pub addresses: Vec<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxOutSetInfoResult {
    pub height: u32,
    pub bestblock: bitcoin::BlockHash,
    pub transactions: u64,
    pub txouts: u32,
    pub bytes_serialized: u64,
    pub hash_serialized: String,
    pub total_amount: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MinerIds {
    pub mined: Vec<MinerId>,
    pub numnotaries: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MinerId {
    pub notaryid: Option<u8>,
    #[serde(rename = "KMDaddress")]
    pub kmd_address: Option<Address>,
    pub pubkey: String, // response could contain `external miners` instead of miner pubkey
    pub blocks: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Notaries {
    pub notaries: Vec<Notary>,
    pub numnotaries: u8,
    pub height: u32,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Notary {
    pub pubkey: bitcoin::PublicKey,
    #[serde(rename = "BTCaddress")]
    pub btc_address: bitcoin::Address,
    #[serde(rename = "KMDaddress")]
    pub kmd_address: Address,
}

// Used for createrawtransaction argument.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRawTransactionInput {
    pub txid: bitcoin::Txid,
    pub vout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
    pub start_time: u64,
    pub addresses: Vec<SnapshotAddress>,
    pub total: f64,
    pub average: f64,
    pub utxos: u64,
    pub total_addresses: u64,
    pub ending_height: u64,
    pub end_time: u64,
    pub ignored_addresses: u32,
    pub skipped_cc_utxos: u32,
    pub cc_utxo_value: f64,
    #[serde(rename = "total_includeCCvouts")]
    pub total_include_ccvouts: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapshotAddress {
    pub addr: String,
    #[serde(deserialize_with = "from_str")]
    pub amount: f64,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenInfo {
    pub result: String,
    pub tokenid: String,
    pub owner: String,
    pub name: String,
    pub supply: u32,
    pub description: String,
    #[serde(default, deserialize_with = "callback_opt")]
    pub data: Option<TokelData>,
    pub version: u32,
    #[serde(rename = "IsMixed")]
    pub is_mixed: String,
    pub height: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokelData {
    pub token_standard_version: u8,
    pub token_id: u8,
    pub token_url: String,
    pub token_royalty_percentage: f32,
    pub token_arbitrary_data_field: Option<String>, // todo this can be its own type and see if it's JSON or other structures.
}

impl TokelData {
    pub fn from_data_string(data: &str) -> Self {
        let evalcode = &data[0..2];
        let evalversion = &data[2..4];

        if evalcode != "f7" {
            panic!("you are not using the correct eval code");
        }

        if evalversion != "01" {
            panic!("this version is not supported");
        }

        let rest = &data[4..];

        let decoded = decode_hex(rest).unwrap();
        let token_id: u8;

        if let Some(pos) = decoded.iter().position(|x| *x == 1) {
            token_id = *decoded.get(pos + 1).unwrap();
        } else {
            panic!("code field id missing");
        }

        let token_url: &str;

        if let Some(pos) = decoded.iter().position(|x| *x == 2) {
            let length = *decoded.get(pos + 1).unwrap();
            let url_bytes = &decoded[pos + 2..=length as usize + 1];
            token_url = str::from_utf8(url_bytes).unwrap();
        } else {
            panic!("code field url missing");
        }

        let token_royalty_percentage: f32;

        if let Some(pos) = decoded.iter().position(|x| *x == 3) {
            let royalty = *decoded.get(pos + 1).unwrap();
            token_royalty_percentage = royalty as f32 / 1000.0;
        } else {
            panic!("code field royalty_percentage missing");
        }

        let token_arbitrary_data_field: Option<String>;

        // 04
        // 05
        // 02 02 ab cd ef
        if let Some(pos) = decoded.iter().position(|x| *x == 4) {
            let length = *decoded.get(pos + 1).unwrap();
            if length > 0 {
                let arbitrary_data_bytes = &decoded[pos + 2..=pos + length as usize + 1];

                if let Ok(valid_string) = str::from_utf8(arbitrary_data_bytes) {
                    token_arbitrary_data_field = Some(String::from(valid_string))
                } else {
                    // todo try to parse this into serde::value if it's JSON
                    // todo what if the arbitrary data field is not utf8?
                    token_arbitrary_data_field = None
                }
            } else {
                token_arbitrary_data_field = None;
            }
        } else {
            token_arbitrary_data_field = None;
        }

        TokelData {
            token_standard_version: str::parse(evalversion).unwrap(),
            token_id,
            token_url: String::from(token_url),
            token_royalty_percentage,
            token_arbitrary_data_field,
        }
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn callback<'de, D>(deserializer: D) -> Result<TokelData, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(TokelData::from_data_string(&s))
}

#[derive(Debug, Deserialize)]
struct WrappedTokelData(#[serde(deserialize_with = "callback")] TokelData);

pub fn callback_opt<'de, D>(deserializer: D) -> Result<Option<TokelData>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<WrappedTokelData>::deserialize(deserializer).map(
        |opt_wrapped: Option<WrappedTokelData>| {
            opt_wrapped.map(|wrapped: WrappedTokelData| wrapped.0)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::TokelData;
    #[test]
    fn non_utf8_arbitrary_data() {
        let tokel_data = TokelData::from_data_string(
            "f70101fe65530600021068747470733a2f2f736974652e6f7267030104050202abcdef",
        );
        assert_eq!(tokel_data.token_id, 254);
        assert_eq!(tokel_data.token_url, "https://si");
        assert!(tokel_data.token_arbitrary_data_field.is_none());
    }
}
