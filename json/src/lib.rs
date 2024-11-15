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
use identity::{IdentityPrimary, IdentityReservation};
use serde::*;
use serde_json::Value;
use std::{collections::HashMap, fmt::Display, str, str::FromStr};

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
pub struct ShieldCoinbaseResult {
    #[serde(rename = "remainingUTXOs")]
    pub remaining_utxos: u32,
    #[serde(rename = "remainingValue")]
    pub remaining_value: f64,
    #[serde(rename = "shieldingUTXOs")]
    pub shielding_utxos: u32,
    #[serde(rename = "shieldingValue")]
    pub shielding_value: f64,
    pub opid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListCurrenciesResult(pub Vec<Currency>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Currency {
    pub currencydefinition: CurrencyDefinition,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyDefinition {
    pub version: u8,
    pub options: u16,
    pub name: String,
    pub currencyid: Address,
    pub parent: Option<Address>,
    pub systemid: Address,
    pub notarizationprotocol: u32,
    pub proofprotocol: u32,
    pub launchsystemid: Option<Address>,
    pub currencyidhex: Option<String>,
    pub fullyqualifiedname: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreAllocation(pub HashMap<String, f64>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCurrencyResult {
    pub version: u16,
    pub options: u16,
    pub name: String,
    pub currencyid: Address,
    pub parent: Option<String>,
    pub systemid: Address,
    pub notarizationprotocol: u8,
    pub proofprotocol: u8,
    pub launchsystemid: Option<Address>,
    pub startblock: u64,
    pub endblock: u64,
    pub preallocations: Option<Vec<PreAllocation>>,
    pub currencies: Option<Vec<Address>>,
    pub weights: Option<Vec<f64>>,
    pub conversions: Option<Vec<f64>>,
    pub initialsupply: Option<f64>,
    pub prelaunchdiscount: Option<f64>,
    pub prelaunchcarveout: Option<f64>,
    pub initialcontributions: Option<Vec<f64>>,
    pub gateway: Option<Address>,
    pub gatewayconverterissuance: Option<f64>,
    pub idregistrationfees: f64,
    pub idreferrallevels: u8,
    pub idimportfees: f64,
    pub currencyidhex: String,
    pub fullyqualifiedname: String,
    pub magicnumber: i64,
    pub currencynames: Option<CurrencyNames>,
    pub definitiontxid: Txid,
    pub definitiontxout: u16,
    pub bestheight: u64,
    pub lastconfirmedheight: Option<u64>,
    pub besttxid: Option<Txid>,
    pub bestcurrencystate: Option<CurrencyState>,
    pub lastconfirmedcurrencystate: Option<CurrencyState>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyNames(pub HashMap<Address, String>);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GetCurrencyStateResult {
    Data(GetCurrencyStateResultInner),
    TotalVolume { totalvolume: f64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCurrencyStateResultInner {
    pub height: u64,
    pub blocktime: u64,
    pub currencystate: CurrencyState,
    pub conversiondata: Option<CurrencyStateConversionData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyState {
    pub flags: u16,
    pub version: u16,
    pub currencyid: Address,
    pub reservecurrencies: Option<Vec<ReserveCurrency>>,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub initialsupply: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub emitted: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub supply: Amount,
    pub currencies: Option<HashMap<Address, CurrencyStateCurrency>>,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub primarycurrencyfees: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub primarycurrencyconversionfees: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub primarycurrencyout: SignedAmount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub preconvertedout: SignedAmount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyStateCurrency {
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub reservein: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub primarycurrencyin: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub reserveout: Amount,
    pub lastconversionprice: f64,
    pub viaconversionprice: f64,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub fees: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub conversionfees: Amount,
    pub priorweights: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyStateConversionData {
    pub volumecurrency: String,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub volumethisinterval: Amount,
    pub volumepairs: Vec<CurrencyStateConversionDataPair>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyStateConversionDataPair {
    pub currency: String,
    pub convertto: String,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub volume: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub open: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub high: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub low: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub close: Amount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReserveCurrency {
    pub currencyid: Address,
    pub weight: f64,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub reserves: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub priceinreserve: Amount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyBalanceResult(pub HashMap<String, f64>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCurrencyConvertersResult {
    #[serde(flatten)]
    pub currencies: HashMap<Address, CurrencyConverterCurrency>,
    pub fullyqualifiedname: String,
    pub height: u64,
    pub output: CurrencyConverterOutput,
    pub lastnotarization: CurrencyConverterNotarization,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyConverterCurrency {
    pub version: u16,
    pub options: u16,
    pub name: String,
    pub currencyid: Address,
    pub parent: Address,
    pub systemid: Address,
    pub notarizationprotocol: u16,
    pub proofprotocol: u16,
    pub launchsystemid: Address,
    pub startblock: u64,
    pub endblock: u64,
    pub currencies: Vec<Address>,
    pub weights: Vec<f32>,
    pub conversions: Vec<f64>,
    pub initialsupply: f64,
    pub prelaunchcarveout: f64,
    pub gateway: Option<Address>,
    pub initialcontributions: Option<Vec<f64>>,
    pub gatewayconverterissuance: Option<f64>,
    pub idregistrationfees: f64,
    pub idreferrallevels: u8,
    pub idimportfees: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyConverterOutput {
    txid: Txid,
    voutnum: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrencyConverterNotarization {
    pub version: u16,
    pub launchcleared: Option<bool>,
    pub launchconfirmed: Option<bool>,
    pub launchcomplete: Option<bool>,
    pub samechain: Option<bool>,
    pub proposer: NotarizationProposer,
    pub currencyid: Address,
    pub notarizationheight: u64,
    pub currencystate: CurrencyState,
    pub prevnotarizationtxid: Txid,
    pub prevnotarizationout: u64,
    pub prevheight: u64,
    pub hashprevcrossnotarization: String,
    pub currencystates: Vec<String>,
    pub proofroots: Vec<ProofRoot>,
    pub nodes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum NotarizationProposer {
    Existing {
        address: Address,
        #[serde(rename = "type")]
        ty: u16,
    },
    NonExisting {
        undefined: String,
        #[serde(rename = "type")]
        ty: u16,
    },
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct NotarizationProposer {
//     pub undefined: Option<String>,
//     pub address: Option<String>,
//     #[serde(rename = "type")]
//     pub ty: u16,
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZOperationStatusResult {
    pub id: String,
    pub status: String,
    pub creation_time: u64,
    pub result: Option<ZOperationStatusResultTxid>,
    pub error: Option<ZOperationStatusResultError>,
    pub execution_secs: Option<f64>,
    pub method: String,
    pub params: Vec<Option<ZOperationStatusResultParam>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZOperationStatusResultError {
    pub code: i32,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZOperationStatusResultParam {
    pub address: String,
    pub amount: f64,
    pub currency: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZOperationStatusResultTxid {
    pub txid: Txid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignRawTransactionResult {
    pub hex: String,
    pub complete: bool,
    pub errors: Option<Vec<SignRawTransactionResultError>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignRawTransactionResultError {
    pub txid: Txid,
    pub vout: u16,
    #[serde(rename = "scriptSig")]
    pub script_sig: String,
    pub sequence: u64,
    pub error: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendCurrencyResult {
    txid: Option<Txid>,
    hextx: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidateAddress {
    #[serde(rename = "isvalid")]
    pub is_valid: bool,
    pub address: Address,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: Script,
    pub segid: u8,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    #[serde(rename = "isscript")]
    pub is_script: bool,
    #[serde(rename = "iswatchonly")]
    pub is_watch_only: bool,
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
    #[serde(rename = "outputIndex")]
    pub output_index: u16,
    pub script: String,
    pub satoshis: u64,
    pub height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddressDelta {
    #[serde(with = "vrsc::util::amount::serde::as_sat")]
    pub satoshis: SignedAmount,
    pub txid: Txid,
    pub index: i32,
    pub blockindex: i32,
    pub height: i64,
    pub spending: bool,
    pub address: Address,
    pub blocktime: u64,
    pub currencyvalues: Option<HashMap<Address, f64>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoinSupply {
    pub result: String,
    pub coin: String,
    pub height: i64,
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
    pub hash: bitcoin::BlockHash,
    #[serde(rename = "validationtype")]
    pub validation_type: ValidationType,
    pub postarget: Option<String>,
    pub poshashbh: Option<String>,
    pub poshashtx: Option<String>,
    pub possourcetxid: Option<Txid>,
    pub possourcevoutnum: Option<u16>,
    pub posrewarddest: Option<Address>,
    pub postxddest: Option<Address>,
    pub confirmations: i32,
    pub size: u32,
    pub height: u64,
    pub version: u32,
    #[serde(rename = "merkleroot")]
    pub merkle_root: bitcoin::TxMerkleNode,
    #[serde(rename = "segid")]
    pub seg_id: i32,
    #[serde(rename = "finalsaplingroot")]
    pub final_sapling_root: String,
    pub tx: Vec<BlockTransaction>,
    // pub tx: Vec<bitcoin::hash_types::Txid>,
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
pub struct BlockTransaction {
    pub txid: Txid,
    pub overwintered: bool,
    pub version: u8,
    pub locktime: u32,
    pub expiryheight: Option<u64>,
    pub vin: Vec<TransactionVin>,
    pub vout: Vec<TransactionVout>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationType {
    Stake,
    Work,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProofRoot {
    pub version: u32,
    #[serde(rename = "type")]
    pub ty: u32,
    pub systemid: Address,
    pub height: u64,
    pub stateroot: String,
    pub blockhash: BlockHash,
    pub power: String,
    pub gasprice: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValuePool {
    pub id: String,
    pub monitored: bool,
    pub chain_value: f64,
    #[serde(rename = "chainValueZat")]
    pub chain_value_sat: u64,
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
    pub eligible_staking_outputs: u32,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub eligible_staking_balance: Amount,
    pub reserve_balance: Option<HashMap<String, f64>>,
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

// TODO deserialize reserve_balance as an Amount
// pub struct WalletInfoReserveBalance { }

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
    pub generated: Option<bool>,
    // confirmations is -1 when not in mempool
    pub confirmations: i32,
    pub blockhash: Option<bitcoin::BlockHash>,
    pub blockindex: Option<u32>,
    pub blocktime: Option<u64>,
    pub expiryheight: Option<u64>,
    pub txid: bitcoin::Txid,
    pub walletconflicts: Vec<Option<bitcoin::Txid>>,
    pub time: u64,
    pub timereceived: u64,
    pub vjoinsplit: Vec<Option<GetTransactionVJoinSplit>>,
    pub details: Vec<GetTransactionDetails>,
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
    pub blockstomaturity: Option<i16>,
    pub amount: f64,
    pub vout: u16,
    pub fee: Option<f64>,
    pub size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GetTransactionDetailsCategory {
    Send,
    Generate,
    Receive,
    Mint,
    Immature,
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
    pub txid: Txid,
    pub overwintered: bool,
    pub version: u32,
    pub versiongroupid: String,
    pub locktime: u64,
    pub expiryheight: u64,
    pub vin: Vec<TransactionVin>,
    pub vout: Vec<TransactionVout>,
    pub vjoinsplit: Vec<GetRawTransactionVJoinSplit>,
    pub blockhash: Option<bitcoin::BlockHash>, // transaction might not be in a block yet
    pub height: Option<i64>,
    pub confirmations: Option<u32>,
    pub time: Option<u64>,
    pub blocktime: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionVin {
    pub txid: Option<bitcoin::Txid>,
    pub vout: Option<u32>,
    pub address: Option<String>,
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<TransactionVinScriptSig>,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc::opt", default)]
    pub value: Option<Amount>,
    #[serde(
        rename = "valueSat",
        with = "vrsc::util::amount::serde::as_sat::opt",
        default
    )]
    pub value_sat: Option<Amount>,
    pub coinbase: Option<String>,
    pub sequence: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionVinScriptSig {
    pub asm: String,
    pub hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionVout {
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub value: Amount,
    #[serde(with = "vrsc::util::amount::serde::as_sat", rename = "valueSat")]
    pub value_sat: Amount,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: TransactionVoutScriptPubKey,
    #[serde(rename = "spentTxId")]
    pub spent_tx_id: Option<Txid>,
    #[serde(rename = "spentIndex")]
    pub spent_index: Option<u32>,
    #[serde(rename = "spentHeight")]
    pub spent_height: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionVoutScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,
    #[serde(rename = "type")]
    pub r#type: String, // cryptocondition, pubkey, scripthash, pubkeyhash TODO
    pub addresses: Option<Vec<Address>>,
    pub identityprimary: Option<IdentityPrimary>,
    #[serde(rename = "identityreservation")]
    pub identity_reservation: Option<IdentityReservation>,
    pub spendableoutput: bool,
    pub reservetransfer: Option<GetRawTransactionScriptPubKeyReserveTransfer>,
    pub crosschainimport: Option<GetRawTransactionScriptPubKeyCrossChainImport>,
    pub reserveoutput: Option<GetRawTransactionScriptPubKeyReserveImport>,
    pub reserve_balance: Option<HashMap<String, f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionScriptPubKeyReserveTransfer {
    pub convert: Option<bool>,
    // #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub currencyvalues: HashMap<Address, f64>,
    pub destination: Value,
    pub destinationcurrencyid: Address,
    pub feecurrencyid: Address,
    pub fees: f64,
    pub flags: i32,
    pub reservetoreserve: Option<bool>,
    pub version: u8,
    pub via: Option<Address>,
    pub spenttxid: Option<Txid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionScriptPubKeyCrossChainImport {
    pub version: u8,
    pub flags: i32,
    pub sourcesystemid: Address,
    pub sourceheight: u64,
    pub importcurrencyid: Address,
    pub valuein: HashMap<Address, f64>,
    pub tokensout: Value,
    pub numoutputs: u32,
    pub hashtransfers: String,
    pub exporttxid: Txid,
    pub spenttxid: Option<Txid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetRawTransactionScriptPubKeyReserveImport {
    version: u8,
    currencyvalues: Value,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockchainInfo {
    // Can be one of main, test or regtest
    pub chain: String,
    pub name: String,
    pub chainid: Address,
    pub blocks: u64,
    // pub synced: bool,
    pub headers: u64,
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
    pub activationheight: u64,
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
    pub height: u64,
    pub version: u32,
    pub merkleroot: String,
    pub time: u64,
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
    pub height: u64,
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
    pub height: u64,
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
    pub height: u64,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    pub id: u32,
    pub addr: String,
    pub addrlocal: String,
    pub services: String,
    pub tls_established: bool,
    pub tls_verified: bool,
    pub lastsend: u64,
    pub lastrecv: u64,
    pub bytessent: u64,
    pub bytesrecv: u64,
    pub conntime: u64,
    pub timeoffset: i32,
    pub pingtime: f64,
    pub version: u32,
    pub subver: String,
    pub inbound: bool,
    pub startingheight: u64,
    pub banscore: u32,
    pub synced_headers: i64,
    pub synced_blocks: i64,
    pub inflight: Vec<u64>,
    pub whitelisted: bool,
}
