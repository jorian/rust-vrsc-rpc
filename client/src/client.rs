use crate::bitcoin;
use crate::bitcoin::BlockHash;
use crate::chain_config::{Auth, ConfigFile};
use crate::error::Error;
use crate::json::identity::*;
use crate::json::*;
use serde_json::{json, Value};
use tracing::*;

use jsonrpc;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::result;
use vrsc::util::address::AddressType;
use vrsc::*;
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonOutPoint {
    pub txid: bitcoin::Txid,
    pub vout: u32,
}

impl From<bitcoin::OutPoint> for JsonOutPoint {
    fn from(o: bitcoin::OutPoint) -> JsonOutPoint {
        JsonOutPoint {
            txid: o.txid,
            vout: o.vout,
        }
    }
}

impl Into<bitcoin::OutPoint> for JsonOutPoint {
    fn into(self) -> bitcoin::OutPoint {
        bitcoin::OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }
}

fn into_json<T>(val: T) -> Result<serde_json::Value>
where
    T: serde::ser::Serialize,
{
    Ok(serde_json::to_value(val)?)
}

/// Shorthand for converting an Option into an Option<serde_json::Value>.
fn opt_into_json<T>(opt: Option<T>) -> Result<serde_json::Value>
where
    T: serde::ser::Serialize,
{
    match opt {
        Some(val) => Ok(into_json(val)?),
        None => Ok(serde_json::Value::Null),
    }
}

/// Shorthand for `serde_json::Value::Null`.
fn null() -> serde_json::Value {
    serde_json::Value::Null
}

/// Shorthand for an empty serde_json::Value array.
fn empty_arr() -> serde_json::Value {
    serde_json::Value::Array(vec![])
}

/// Shorthand for an empty serde_json object.
// fn empty_obj() -> serde_json::Value {
//     serde_json::Value::Object(Default::default())
// }

/// Handle default values in the argument list
///
/// Substitute `Value::Null`s with corresponding values from `defaults` table,
/// except when they are trailing, in which case just skip them altogether
/// in returned list.
///
/// Note, that `defaults` corresponds to the last elements of `args`.
///
/// ```norust
/// arg1 arg2 arg3 arg4
///           def1 def2
/// ```
///
/// Elements of `args` without corresponding `defaults` value, won't
/// be substituted, because they are required.
fn handle_defaults<'a, 'b>(
    args: &'a mut [serde_json::Value],
    defaults: &'b [serde_json::Value],
) -> &'a [serde_json::Value] {
    assert!(args.len() >= defaults.len());

    // Pass over the optional arguments in backwards order, filling in defaults after the first
    // non-null optional argument has been observed.
    let mut first_non_null_optional_idx = None;
    for i in 0..defaults.len() {
        let args_i = args.len() - 1 - i;
        let defaults_i = defaults.len() - 1 - i;
        if args[args_i] == serde_json::Value::Null {
            if first_non_null_optional_idx.is_some() {
                if defaults[defaults_i] == serde_json::Value::Null {
                    panic!("Missing `default` for argument idx {}", args_i);
                }
                args[args_i] = defaults[defaults_i].clone();
            }
        } else if first_non_null_optional_idx.is_none() {
            first_non_null_optional_idx = Some(args_i);
        }
    }

    let required_num = args.len() - defaults.len();

    if let Some(i) = first_non_null_optional_idx {
        &args[..i + 1]
    } else {
        &args[..required_num]
    }
}

#[derive(Debug)]
pub struct Client {
    client: jsonrpc::client::Client,
}

impl Client {
    pub fn chain(testnet: bool, currencyidhex: &str, auth: Auth) -> Result<Self> {
        match auth {
            Auth::ConfigFile => {
                let config = ConfigFile::pbaas(testnet, currencyidhex)?;
                Ok(Client {
                    client: jsonrpc::client::Client::simple_http(
                        &format!("http://127.0.0.1:{}", config.rpcport),
                        Some(config.rpcuser),
                        Some(config.rpcpassword),
                    )
                    .unwrap(),
                })
            }
            Auth::UserPass(url, rpcuser, rpcpassword) => Ok(Client {
                client: jsonrpc::client::Client::simple_http(
                    &url,
                    Some(rpcuser),
                    Some(rpcpassword),
                )
                .unwrap(),
            }),
        }
    }

    pub fn vrsc(testnet: bool, auth: Auth) -> Result<Self> {
        match auth {
            Auth::ConfigFile => {
                let config = ConfigFile::vrsc(testnet)?;
                Ok(Client {
                    client: jsonrpc::client::Client::simple_http(
                        &format!("http://127.0.0.1:{}", config.rpcport),
                        Some(config.rpcuser),
                        Some(config.rpcpassword),
                    )
                    .unwrap(),
                })
            }
            Auth::UserPass(url, rpcuser, rpcpassword) => Ok(Client {
                client: jsonrpc::client::Client::simple_http(
                    &url,
                    Some(rpcuser),
                    Some(rpcpassword),
                )
                .unwrap(),
            }),
        }
    }
}

impl Default for Client {
    /// Creates a default Verus client based on parameters found in the VRSC.conf file.
    /// Panics if
    /// - `VRSC.conf` does not exist
    /// - one of rpcport, rpcuser or rpcpassword is not found in VRSC.conf
    fn default() -> Self {
        if let Ok(config) = ConfigFile::vrsc(false) {
            Client {
                client: jsonrpc::client::Client::simple_http(
                    &format!("http://127.0.0.1:{}", config.rpcport),
                    Some(config.rpcuser),
                    Some(config.rpcpassword),
                )
                .unwrap(),
            }
        } else {
            panic!("no valid Verus configuration found")
        }
    }
}

impl RpcApi for Client {
    fn call<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        cmd: &str,
        args: &[serde_json::Value],
    ) -> Result<T> {
        let raw_args: Vec<_> = args
            .iter()
            .map(|a| {
                let json_string = serde_json::to_string(a)?;
                serde_json::value::RawValue::from_string(json_string)
            })
            .map(|a| a.map_err(|e| Error::Json(e)))
            .collect::<Result<Vec<_>>>()?;
        let req = self.client.build_request(&cmd, &raw_args);

        debug!("{:#?}", &req);

        let resp = self.client.send_request(req).map_err(Error::from);

        debug!("{:#?}", &resp);

        Ok(resp?.result()?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AddressList {
    pub addresses: Vec<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListCurrenciesQueryObject {
    pub systemtype: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendCurrencyOutput<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<&'a str>,
    #[serde(with = "vrsc::util::amount::serde::as_vrsc")]
    pub amount: Amount,
    pub address: String,
}

impl<'a> SendCurrencyOutput<'a> {
    pub fn new(currency: Option<&'a str>, amount: &Amount, address: &str) -> Self {
        SendCurrencyOutput {
            currency: currency,
            amount: amount.clone(),
            address: address.to_string(),
        }
    }
}

impl<'a> From<(&Address, &Amount)> for SendCurrencyOutput<'a> {
    fn from((address, amount): (&Address, &Amount)) -> Self {
        Self {
            currency: None,
            amount: *amount,
            address: address.to_string(),
        }
    }
}

// This trait is to be implemented by an implementation of a client, and only the `call` method
// is to be implemented.
// All the other methods are methods that a client can call, which in turn do RPCs to the coin daemon.
// the `for` keyword used in serde is done to let the serde deserializer determine the lifetime of
// anything that is put out, in contrast with letting the function caller determine the lifetime.
// (Higher-Ranked Trait Bounds)
pub trait RpcApi: Sized {
    fn call<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        cmd: &str,
        args: &[serde_json::Value],
    ) -> Result<T>;

    fn rescan_from_height(&self, height: u64) -> Result<()> {
        self.call("rescanfromheight", &[height.into()])
    }

    fn get_currency(&self, currency: &str) -> Result<GetCurrencyResult> {
        self.call("getcurrency", &[into_json(currency)?])
    }

    // get_currency_state() for
    fn get_currency_state(&self, currency: &str) -> Result<Vec<GetCurrencyStateResult>> {
        self.call("getcurrencystate", &[into_json(currency)?])
    }
    fn get_vdxf_id(&self, uri: &str, options: Option<Value>) -> Result<GetVDXFIdResult> {
        self.call("getvdxfid", &[uri.into(), opt_into_json(options)?])
    }

    fn get_identities_with_address(
        &self,
        address: &str,
        fromheight: Option<u64>,
        toheight: Option<u64>,
        unspent: Option<bool>,
    ) -> Result<IdentitiesWithAddressResult> {
        let input = json!({
            "address": address,
            "fromheight": fromheight.unwrap_or(0),
            "toheight": toheight.unwrap_or(0),
            "unspent": unspent.unwrap_or(false)
        });

        self.call("getidentitieswithaddress", &[input])
    }

    fn list_currencies(&self, system_type: Option<&str>) -> Result<ListCurrenciesResult> {
        if let Some(systemtype) = system_type {
            self.call(
                "listcurrencies",
                &[into_json(ListCurrenciesQueryObject {
                    systemtype: String::from(systemtype),
                })?],
            )
        } else {
            self.call("listcurrencies", &[])
        }
    }

    fn z_get_operation_status(
        &self,
        opid: Vec<&str>,
    ) -> Result<Vec<Option<ZOperationStatusResult>>> {
        self.call("z_getoperationstatus", &[into_json(opid)?])
    }

    // fn z_shield_coinbase(
    //     &self,
    //     from: &str,
    //     to_zaddress: &str,
    //     fee: Option<f64>,
    //     limit: Option<u8>,
    // ) -> Result<ShieldCoinbaseResult> {
    //     self.call(
    //         "z_shieldcoinbase",
    //         &[
    //             from.into(),
    //             to_zaddress.into(),
    //             opt_into_json(fee)?,
    //             opt_into_json(limit)?,
    //         ],
    //     )
    // }

    // the from address can be sapling, an id, an actual address or a wildcard address
    fn send_currency(
        &self,
        from: &str,
        outputs: Vec<SendCurrencyOutput>,
        _minconf: Option<u16>,
        _fee_amount: Option<f64>,
    ) -> Result<String> {
        self.call("sendcurrency", &[from.into(), into_json(outputs)?])
    }

    fn validate_address<S>(&self, address: S) -> Result<ValidateAddress>
    where
        S: Into<String>,
    {
        self.call("validateaddress", &[address.into().into()])
    }

    fn get_mining_info(&self) -> Result<MiningInfo> {
        self.call("getmininginfo", &[])
    }

    fn get_address_utxos(&self, addresses: Vec<Address>) -> Result<Vec<AddressUtxos>> {
        self.call("getaddressutxos", &[into_json(AddressList { addresses })?])
    }

    fn get_address_deltas(
        &self,
        addresses: &[&Address],
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<Vec<AddressDelta>> {
        let input = json!({
            "addresses": into_json(addresses)?,
            "start": start.unwrap_or(0),
            "end": end.unwrap_or(9999999)
        });

        self.call("getaddressdeltas", &[input])
    }

    fn get_address_balance(&self, addresses: Vec<Address>) -> Result<Vec<AddressUtxos>> {
        self.call("getaddressdeltas", &[into_json(AddressList { addresses })?])
    }
    // Identity

    /// Simplest way of getting an identity
    /// TODO add i-address, height, txproof, txproofheight
    fn get_identity(&self, name: &str) -> Result<Identity> {
        self.call("getidentity", &[name.into()])
    }

    // TODO: RPC returns nothing on empty list
    // see: https://github.com/VerusCoin/VerusCoin/issues/381
    fn list_identities(&self) -> Result<Vec<Identity>> {
        self.call("listidentities", &[])
    }

    fn recoveridentity(&self) -> Result<()> {
        unimplemented!()
    }
    fn registeridentity(
        &self,
        namecommitment: &NameCommitment,
        addresses: &Vec<Address>,
        minimum_signatures: Option<u8>,
        private_address: Option<String>,
        currency_name: Option<String>,
        content_map: Option<serde_json::Value>,
    ) -> Result<bitcoin::Txid> {
        #[derive(Serialize)]
        struct Argument<'a> {
            txid: bitcoin::Txid,
            namereservation: NameReservation,
            identity: Identity<'a>,
        }

        #[derive(Serialize)]
        struct Identity<'a> {
            name: &'a str,
            primaryaddresses: &'a Vec<Address>,
            #[serde(skip_serializing_if = "Option::is_none")]
            minimumsignatures: Option<u8>,
            #[serde(skip_serializing_if = "Option::is_none")]
            privateaddress: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            revocationauthority: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            recoveryauthority: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            contentmap: Option<serde_json::Value>,
        }

        let identity_name = match currency_name {
            Some(currency) => format!("{}.{}@", &namecommitment.namereservation.name, currency),
            None => namecommitment.namereservation.name.clone(),
        };

        self.call(
            "registeridentity",
            &[into_json(Argument {
                txid: namecommitment.txid,
                namereservation: namecommitment.namereservation.clone(),
                identity: Identity {
                    name: &identity_name,
                    primaryaddresses: addresses,
                    minimumsignatures: minimum_signatures,
                    privateaddress: private_address,
                    recoveryauthority: None,
                    revocationauthority: None,
                    contentmap: content_map,
                },
            })?],
        )
    }

    // a referral can either be an identity name (identity@) or an identity address (address that starts with i)
    // TODO
    fn registernamecommitment(
        &self,
        name: &str,
        controll_address: &Address,
        referral: Option<String>,
        parentnameorid: Option<String>,
    ) -> Result<NameCommitment> {
        self.call(
            "registernamecommitment",
            &[
                name.into(),
                controll_address.to_string().into(),
                opt_into_json(referral)?,
                opt_into_json(parentnameorid)?,
            ],
        )

        // if let Some(referral) = referral {
        //     self.call(
        //         "registernamecommitment",
        //         &[
        //             name.into(),
        //             controll_address.to_string().into(),
        //             referral.into(),
        //             opt_into_json(parentnameorid)?,
        //         ],
        //     )
        // } else {
        //     self.call(
        //         "registernamecommitment",
        //         &[name.into(), controll_address.to_string().into()],
        //     )
        // }
    }
    fn revokeidentity(&self) -> Result<()> {
        unimplemented!()
    }
    fn setidentitytimelock(&self) -> Result<()> {
        unimplemented!()
    }
    fn signfile(&self) -> Result<()> {
        unimplemented!()
    }
    fn signmessage(&self) -> Result<()> {
        unimplemented!()
    }
    fn updateidentity(&self) -> Result<()> {
        unimplemented!()
    }
    fn verifyfile(&self) -> Result<()> {
        unimplemented!()
    }
    fn verifyhash(&self) -> Result<()> {
        unimplemented!()
    }
    fn verifymessage(&self) -> Result<()> {
        unimplemented!()
    }

    fn coin_supply(&self, height: &str) -> Result<CoinSupply> {
        // TODO why is height a str?
        self.call("coinsupply", &[height.into()])
    }
    fn get_best_blockhash(&self) -> Result<bitcoin::BlockHash> {
        self.call("getbestblockhash", &[])
    }

    // Marketplace
    fn closeoffers(&self) -> Result<()> {
        unimplemented!()
    } // ('["offer1_txid", "offer2_txid", ...]') (transparentorprivatefundsdestination) (privatefundsdestination)

    fn get_offers(
        &self,
        currency_or_id: &str,
        is_currency: bool,
        with_raw_tx: bool,
    ) -> Result<HashMap<String, Vec<MarketplaceOffer>>> {
        self.call(
            "getoffers",
            &[
                currency_or_id.into(),
                is_currency.into(),
                with_raw_tx.into(),
            ],
        )
    }

    fn listopenoffers(&self) -> Result<()> {
        unimplemented!()
    } // (unexpired) (expired)'
    fn makeoffer(&self) -> Result<()> {
        unimplemented!()
    } // fromaddress '{"changeaddress":"transparentoriaddress", "expiryheight":blockheight, "offer":{"currency":"anycurrency", "amount":...} | {"identity":"idnameoriaddress",...}', "for":{"address":..., "currency":"anycurrency", "amount":...} | {"name":"identityforswap","parent":"parentid","primaryaddresses":["R-address(s)"],"minimumsignatures":1,...}}' (returntx) (feeamount)
    fn takeoffer(&self) -> Result<()> {
        unimplemented!()
    } // fromaddress '{"txid":"txid" | "tx":"hextx", "changeaddress":"transparentoriaddress", "deliver":"fullidnameoriaddresstodeliver" | {"currency":"currencynameorid","amount":n}, "accept":{"address":"addressorid","currency":"currencynameorid","amount"} | {identitydefinition}}' (returntx) (feeamount)

    /// Get a block, based on its hash (later on: and height todo).
    fn get_block(&self, hash: &bitcoin::BlockHash, verbosity: u8) -> Result<Block> {
        let val = serde_json::to_value(hash)?;

        self.call("getblock", &[val, verbosity.into()])
        // the BTC rpc library explicitly validates the bytes that are returned from the daemon.

        // let hex: String = self.call("getblock", &[val])?;
        // let bytes: Vec<u8> = FromHex::from_hex(&hex)?;
        // let deserialized = bitcoin::consensus::encode::deserialize(&bytes)?;
        //
        // Ok(deserialized)
        // fetch the hex
        // make it a Vec<u8>
        // that data needs to be consensus deserialized, to make sure it is a valid hash.
        // into_json()
    }

    fn get_block_by_height(&self, height: u64, verbosity: u8) -> Result<Block> {
        // let val = serde_json::to_value(hash)?;

        self.call("getblock", &[height.to_string().into(), verbosity.into()])
        // the BTC rpc library explicitly validates the bytes that are returned from the daemon.

        // let hex: String = self.call("getblock", &[val])?;
        // let bytes: Vec<u8> = FromHex::from_hex(&hex)?;
        // let deserialized = bitcoin::consensus::encode::deserialize(&bytes)?;
        //
        // Ok(deserialized)
        // fetch the hex
        // make it a Vec<u8>
        // that data needs to be consensus deserialized, to make sure it is a valid hash.
        // into_json()
    }

    fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        self.call("getblockchaininfo", &[])
    }
    fn get_block_count(&self) -> Result<u32> {
        self.call("getblockcount", &[])
    }

    fn get_block_hash(&self, height: u64) -> Result<bitcoin::BlockHash> {
        self.call("getblockhash", &[height.into()])
    }

    fn get_blockhashes(&self) -> Result<()> {
        unimplemented!()
    }
    fn get_blockheader_verbose(&self, hash: &bitcoin::BlockHash) -> Result<BlockHeader> {
        self.call("getblockheader", &[into_json(hash)?, into_json(true)?])
    }
    fn get_blockheader(&self, hash: &bitcoin::BlockHash) -> Result<String> {
        self.call("getblockheader", &[into_json(hash)?, into_json(false)?])
    }
    fn get_chaintips(&self) -> Result<ChainTips> {
        self.call("getchaintips", &[])
    }
    fn get_chain_tx_stats(
        &self,
        n: Option<u32>,
        blockhash: Option<bitcoin::BlockHash>,
    ) -> Result<ChainTxStats> {
        let mut args = [opt_into_json(n)?, opt_into_json(blockhash)?];

        let defaults = [null(), null()];
        self.call("getchaintxstats", handle_defaults(&mut args, &defaults))
    }
    fn get_difficulty(&self) -> Result<f64> {
        self.call("getdifficulty", &[])
    }
    fn get_last_segid_stakes(&self) -> Result<()> {
        unimplemented!()
    }
    fn get_mempool_info(&self) -> Result<MempoolInfo> {
        self.call("getmempoolinfo", &[])
    }

    fn get_peer_info(&self) -> Result<Vec<PeerInfo>> {
        self.call("getpeerinfo", &[])
    }

    fn get_raw_mempool(&self) -> Result<Vec<bitcoin::Txid>> {
        self.call("getrawmempool", &[])
    }

    fn get_raw_mempool_verbose(&self) -> Result<RawMempool> {
        self.call("getrawmempool", &[into_json(true)?])
    }

    fn get_spent_info(&self, _txid: bitcoin::Txid, _index: u32) -> Result<SpentInfoResult> {
        // let mut hashmap: HashMap<String, String> = HashMap::new();
        // hashmap.insert(String::from("txid"), txid.to_string());
        // hashmap.insert(String::from("index"), index.to_string());

        // let args = [into_json(hashmap)?];

        // self.call("getspentinfo", &args)

        // TODO the getspentinfo call does not work

        unimplemented!()
    }
    fn get_txout(
        &self,
        txid: &bitcoin::Txid,
        n_vout: u32,
        include_mempool: Option<bool>,
    ) -> Result<TxOutResult> {
        let mut args = [
            into_json(txid.to_string())?,
            into_json(n_vout)?,
            opt_into_json(include_mempool)?,
        ];

        let defaults = [into_json(false)?];

        self.call("gettxout", handle_defaults(&mut args, &defaults))
    }
    fn get_txout_proof(
        &self,
        txids: Vec<bitcoin::Txid>,
        blockhash: Option<bitcoin::BlockHash>,
    ) -> Result<String> {
        let mut args = [into_json(txids)?, opt_into_json(blockhash)?];

        self.call("gettxoutproof", handle_defaults(&mut args, &[null()]))
    }
    fn get_txout_set_info(&self) -> Result<TxOutSetInfoResult> {
        self.call("gettxoutsetinfo", &[])
    }
    fn kvsearch(&self) -> Result<()> {
        unimplemented!()
    }
    fn kvupdate(&self) -> Result<()> {
        unimplemented!()
    }
    fn miner_ids(&self, height: u64) -> Result<MinerIds> {
        self.call("minerids", &[into_json(height.to_string())?])
    }
    fn notaries(&self, height: u64) -> Result<Notaries> {
        self.call("notaries", &[into_json(height.to_string())?])
    }
    fn verify_chain(&self, checklevel: Option<u8>, numblocks: Option<u32>) -> Result<bool> {
        let mut args = [opt_into_json(checklevel)?, opt_into_json(numblocks)?];

        let defaults = [into_json(3)?, into_json(288)?];

        self.call("verifychain", handle_defaults(&mut args, &defaults))
    }
    fn verify_txout_proof(&self, proof: &str) -> Result<Vec<Option<bitcoin::Txid>>> {
        self.call("verifytxoutproof", &[into_json(proof)?])
    }

    fn createrawtransaction(
        &self,
        inputs: &[CreateRawTransactionInput],
        outputs: &HashMap<String, Amount>,
        locktime: Option<i64>,
        expiryheight: Option<u64>,
    ) -> Result<String> {
        let outputs_converted = serde_json::Map::from_iter(
            outputs
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::from(v.as_vrsc()))),
        );
        let mut args = [
            into_json(inputs)?,
            into_json(outputs_converted)?,
            opt_into_json(locktime)?,
            opt_into_json(expiryheight)?,
        ];
        let defaults = [into_json(0i64)?, null()];
        self.call(
            "createrawtransaction",
            handle_defaults(&mut args, &defaults),
        )
    }
    fn decoderawtransaction(&self) -> Result<()> {
        unimplemented!()
    }
    fn decodescript(&self) -> Result<()> {
        unimplemented!()
    }
    fn fundrawtransaction(&self) -> Result<()> {
        unimplemented!()
    }
    fn getrawtransaction(&self) -> Result<()> {
        unimplemented!()
    }
    fn sendrawtransaction(&self, signed_hex: &str) -> Result<String> {
        self.call("sendrawtransaction", &[signed_hex.into()])
    }

    fn signrawtransaction(&self, hex: &str) -> Result<SignRawTransactionResult> {
        self.call("signrawtransaction", &[hex.into()])
    }

    fn get_raw_transaction_verbose(
        &self,
        txid: &bitcoin::Txid,
    ) -> Result<GetRawTransactionResultVerbose> {
        self.call("getrawtransaction", &[into_json(txid)?, 1.into()])
    }

    fn get_raw_transaction(&self, txid: &bitcoin::Txid) -> Result<GetRawTransactionResult> {
        self.call("getrawtransaction", &[into_json(txid)?, 0.into()])
    }

    fn ping(&self) -> Result<()> {
        self.call("ping", &[])
    }

    // Label is deprecated and thus not used in the method call.
    // Todo keys are either an address or a pubkey.
    fn add_multi_sig_address(&self, n_required: u8, keys: &[PubkeyOrAddress]) -> Result<String> {
        // maximum of 15 in a msig.
        if n_required > 15 {
            return Err(Error::VRSCError(String::from(
                "No more than 15 signers in a msig allowed",
            )));
        }

        self.call("addmultisigaddress", &[n_required.into(), into_json(keys)?])
    }

    fn backup_wallet(&self, destination: &str) -> Result<PathBuf> {
        self.call("backupwallet", &[destination.into()])
            .map(|path: String| PathBuf::from(&path))
    }

    fn clean_wallet_transactions(&self) -> Result<CleanedWalletTransactions> {
        self.call("cleanwallettransactions", &[])
    }

    fn convert_passphrase(&self, passphrase: &str) -> Result<ConvertedPassphrase> {
        self.call("convertpassphrase", &[passphrase.into()])
    }

    fn dump_privkey(&self, address: Address) -> Result<PrivateKey> {
        match address.addr_type {
            AddressType::Shielded => {
                return Err(Error::VRSCError(String::from(
                    "no support for shielded addresses for this call",
                )))
            }
            _ => {}
        }
        self.call("dumpprivkey", &[address.to_string().into()])
    }

    fn get_balance(
        &self,
        minconf: Option<usize>,
        include_watchonly: Option<bool>,
    ) -> Result<Amount> {
        let mut args = [opt_into_json(minconf)?, opt_into_json(include_watchonly)?];
        Ok(Amount::from_vrsc(self.call(
            "getbalance",
            handle_defaults(&mut args, &[0.into(), null()]),
        )?)?)
    }

    fn get_new_address(&self) -> Result<Address> {
        self.call("getnewaddress", &[])
    }

    fn get_raw_change_address(&self) -> Result<Address> {
        self.call("getrawchangeaddress", &[])
    }

    fn get_received_by_address(&self, address: &Address, minconf: Option<usize>) -> Result<Amount> {
        let mut args = [address.to_string().into(), opt_into_json(minconf)?];
        Ok(Amount::from_vrsc(self.call(
            "getreceivedbyaddress",
            handle_defaults(&mut args, &[1.into()]),
        )?)?)
    }

    fn get_transaction(
        &self,
        txid: &bitcoin::Txid,
        include_watch_only: Option<bool>,
    ) -> Result<GetTransactionResult> {
        let mut args = [into_json(txid)?, opt_into_json(include_watch_only)?];
        self.call("gettransaction", handle_defaults(&mut args, &[null()]))
    }

    fn import_address(
        &self,
        address: &Address,
        label: Option<&str>,
        rescan: Option<bool>,
    ) -> Result<()> {
        let mut args = [
            address.to_string().into(),
            opt_into_json(label)?,
            opt_into_json(rescan)?,
        ];
        self.call(
            "importaddress",
            handle_defaults(&mut args, &[into_json("")?, null()]),
        )
    }

    fn import_private_key(
        &self,
        privkey: &PrivateKey,
        label: Option<&str>,
        rescan: Option<bool>,
    ) -> Result<Address> {
        let mut args = [
            privkey.to_string().into(),
            opt_into_json(label)?,
            opt_into_json(rescan)?,
        ];
        self.call(
            "importprivkey",
            handle_defaults(&mut args, &[into_json("")?, null()]),
        )
    }

    fn keypool_refill(&self, newsize: Option<usize>) -> Result<()> {
        let mut args = [opt_into_json(newsize)?];
        self.call("keypoolrefill", handle_defaults(&mut args, &[null()]))
    }

    fn list_lock_unspent(&self) -> Result<Vec<ListLockUnspentResult>> {
        self.call("listlockunspent", &[])
    }

    fn list_received_by_address(
        &self,
        minconf: Option<usize>,
        include_empty: Option<bool>,
        include_watch_only: Option<bool>,
    ) -> Result<Vec<ListReceivedByAddressResult>> {
        let mut args = [
            opt_into_json(minconf)?,
            opt_into_json(include_empty)?,
            opt_into_json(include_watch_only)?,
        ];
        self.call(
            "listreceivedbyaddress",
            handle_defaults(&mut args, &[1.into(), null(), null()]),
        )
    }

    fn list_since_block(
        &self,
        blockhash: Option<&BlockHash>,
        target_confirmations: Option<usize>,
        include_watch_only: Option<bool>,
    ) -> Result<ListSinceBlockResult> {
        let mut args = [
            opt_into_json(blockhash)?,
            opt_into_json(target_confirmations)?,
            opt_into_json(include_watch_only)?,
        ];
        self.call(
            "listsinceblock",
            handle_defaults(&mut args, &[null(), 1.into(), null()]),
        )
    }

    fn list_transactions(
        &self,
        count: Option<u32>,
        from: Option<u32>,
        include_watch_only: Option<bool>,
    ) -> Result<Vec<ListTransactionsResult>> {
        let mut args = [
            opt_into_json(count)?,
            opt_into_json(from)?,
            opt_into_json(include_watch_only)?,
        ];
        self.call(
            "listtransactions",
            handle_defaults(&mut args, &[10.into(), 0.into(), null()]),
        )
    }

    fn list_unspent(
        &self,
        minconf: Option<usize>,
        maxconf: Option<usize>,
        addresses: Option<&Vec<Address>>,
    ) -> Result<Vec<ListUnspentResult>> {
        let mut args = [
            opt_into_json(minconf)?,
            opt_into_json(maxconf)?,
            opt_into_json(addresses)?,
        ];
        let defaults = [into_json(0)?, into_json(9999999)?, empty_arr()];
        self.call("listunspent", handle_defaults(&mut args, &defaults))
    }

    /// To unlock, use [unlock_unspent].
    fn lock_unspent(&self, outputs: &[bitcoin::OutPoint]) -> Result<bool> {
        let outputs: Vec<_> = outputs
            .into_iter()
            .map(|o| serde_json::to_value(JsonOutPoint::from(*o)).unwrap())
            .collect();
        self.call("lockunspent", &[false.into(), outputs.into()])
    }

    fn unlock_unspent(&self, outputs: &[bitcoin::OutPoint]) -> Result<bool> {
        let outputs: Vec<_> = outputs
            .into_iter()
            .map(|o| serde_json::to_value(JsonOutPoint::from(*o)).unwrap())
            .collect();
        self.call("lockunspent", &[true.into(), outputs.into()])
    }

    fn opreturn_burn(
        &self,
        amount: f64,
        hex_str: &str,
        txfee: Option<f64>,
    ) -> Result<OpReturnBurnResult> {
        let mut args = [amount.into(), hex_str.into(), opt_into_json(txfee)?];
        self.call(
            "opreturn_burn",
            handle_defaults(&mut args, &[into_json(0.0001)?]),
        )
    }

    fn resend_wallet_transactions(&self) -> Result<Vec<bitcoin::Txid>> {
        self.call("resendwallettransactions", &[])
    }

    fn send_many(
        &self,
        amounts: &HashMap<Address, Amount>,
        minconf: Option<u16>,
        comment: Option<&str>,
        subtract_fee_from_amount: Option<&Vec<Address>>,
    ) -> Result<bitcoin::Txid> {
        let amounts_converted = serde_json::Map::from_iter(
            amounts
                .iter()
                .map(|(k, v)| (k.to_string().clone(), serde_json::Value::from(v.as_vrsc()))),
        );
        let mut args = [
            "".into(),
            into_json(amounts_converted)?,
            opt_into_json(minconf)?,
            opt_into_json(comment)?,
            opt_into_json(subtract_fee_from_amount)?,
        ];
        let defaults = [
            into_json(1)?,
            into_json("")?,
            into_json(Vec::<Address>::new())?,
        ];

        self.call("sendmany", handle_defaults(&mut args, &defaults))
    }

    fn send_to_address(
        &self,
        address: &Address,
        amount: &Amount,
        minconf: Option<u32>,
        comment: Option<&str>,
        comment_to: Option<&str>,
        subtract_fee_from_amount: Option<bool>,
    ) -> Result<bitcoin::Txid> {
        let mut args = [
            into_json(address.to_string())?,
            into_json(amount.as_vrsc())?,
            opt_into_json(minconf)?,
            opt_into_json(comment)?,
            opt_into_json(comment_to)?,
            opt_into_json(subtract_fee_from_amount)?,
        ];
        let defaults = [
            into_json(1)?,
            into_json("")?,
            into_json("")?,
            into_json(false)?,
        ];
        self.call("sendtoaddress", handle_defaults(&mut args, &defaults))
    }

    // fn set_pubkey(&self, pubkey: &komodo::PublicKey) -> Result<SetPubkeyResult> {
    //     self.call("setpubkey", &[into_json(pubkey.to_string())?])
    // }

    fn sign_message(&self, address: &Address, message: &str) -> Result<String> {
        self.call("signmessage", &[address.to_string().into(), message.into()])
    }

    fn get_unconfirmed_balance(&self) -> Result<f64> {
        self.call("getunconfirmedbalance", &[])
    }

    fn get_wallet_info(&self) -> Result<WalletInfo> {
        self.call("getwalletinfo", &[])
    }

    fn set_tx_fee(&self, amount: f64) -> Result<bool> {
        self.call("settxfee", &[amount.into()])
    }

    fn get_snapshot(&self, top: Option<String>) -> Result<Snapshot> {
        let mut args = [opt_into_json(top)?];
        self.call("getsnapshot", handle_defaults(&mut args, &[null()]))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{Auth, Client, ConfigFile};

    // todo https://github.com/iredelmeier/filesystem-rs/blob/master/src/lib.rs

    #[test]
    fn get_config() {
        let config_file = ConfigFile::vrsc(false).unwrap();
        println!("{:#?}", &config_file);

        let client = Client::vrsc(false, Auth::ConfigFile);
        assert!(client.is_ok());

        let client = Client::vrsc(
            false,
            Auth::UserPass(
                "http://127.0.0.1:27777".to_string(),
                "1kj23k1l23".to_string(),
                "5jkhkjhl5".to_string(),
            ),
        );
        assert!(client.is_ok());
    }
}
