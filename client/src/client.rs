use crate::error::Error;
use jsonrpc;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::result;

use crate::bitcoin::BlockHash;
use crate::chain_config::{Auth, ConfigFile};
use crate::json::identity::Identity;
use crate::json::vrsc::util::address::AddressType;
use crate::json::vrsc::util::amount::Amount;
use crate::json::*;
use crate::{bitcoin, json};

use vrsc_rpc_json::vrsc::{Address, PrivateKey};
use vrsc_rpc_json::GetTransactionResult;

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

pub struct Client {
    client: jsonrpc::client::Client,
}

impl Client {
    pub fn chain(name: &str, auth: Auth) -> Result<Self> {
        match auth {
            Auth::ConfigFile => {
                let config = ConfigFile::new(name)?;
                Ok(Client {
                    client: jsonrpc::client::Client::new(
                        format!("http://127.0.0.1:{}", config.rpcport),
                        Some(config.rpcuser),
                        Some(config.rpcpassword),
                    ),
                })
            }
            Auth::UserPass(url, rpcuser, rpcpassword) => Ok(Client {
                client: jsonrpc::client::Client::new(url, Some(rpcuser), Some(rpcpassword)),
            }),
        }
    }
}

impl Default for Client {
    /// Creates a default VerusClient based on parameters found in the VRSC.conf file in `$HOME/.komodo/VRSC/VRSC.conf`
    /// Panics if
    /// - $HOME/.komodo/VRSC/VRSC.conf` does not exist
    /// - one of rpcport, rpcuser or rpcpassword is not found in VRSC.conf
    fn default() -> Self {
        if let Ok(config) = ConfigFile::new("VRSC") {
            Client {
                client: jsonrpc::client::Client::new(
                    format!("http://127.0.0.1:{}", config.rpcport),
                    Some(config.rpcuser),
                    Some(config.rpcpassword),
                ),
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
        let req = self.client.build_request(&cmd, &args);

        // dbg!(&req);

        let resp = self.client.send_request(&req).map_err(Error::from);

        dbg!(&resp);

        Ok(resp?.into_result()?)
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

    // Identity

    /// Simplest way of getting an identity
    /// TODO add i-address, height, txproof, txproofheight
    fn get_identity(&self, name: &str) -> Result<Identity> {
        self.call("getidentity", &[name.into()])
    }

    fn list_identities(&self) -> Result<Vec<Identity>> {
        self.call("listidentities", &[])
    }

    fn recoveridentity(&self) -> Result<()> {
        unimplemented!()
    }
    fn registeridentity(&self) -> Result<()> {
        unimplemented!()
    }
    fn registernamecommitment(&self) -> Result<()> {
        unimplemented!()
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

    /// Get a block, based on its hash (later on: and height todo).
    fn get_block(&self, hash: &bitcoin::BlockHash) -> Result<json::Block> {
        let val = serde_json::to_value(hash)?;

        self.call("getblock", &[val])
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
        txid: bitcoin::Txid,
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
        inputs: &[json::CreateRawTransactionInput],
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
    fn sendrawtransaction(&self) -> Result<()> {
        unimplemented!()
    }
    fn signrawtransaction(&self) -> Result<()> {
        unimplemented!()
    }

    fn get_raw_transaction_verbose(
        &self,
        txid: &bitcoin::Txid,
    ) -> Result<json::GetRawTransactionResultVerbose> {
        self.call("getrawtransaction", &[into_json(txid)?, 1.into()])
    }

    fn get_raw_transaction(&self, txid: &bitcoin::Txid) -> Result<json::GetRawTransactionResult> {
        self.call("getrawtransaction", &[into_json(txid)?, 0.into()])
    }

    fn ping(&self) -> Result<()> {
        self.call("ping", &[])
    }

    // Label is deprecated and thus not used in the method call.
    // Todo keys are either an address or a pubkey.
    fn add_multi_sig_address(
        &self,
        n_required: u8,
        keys: &[json::PubkeyOrAddress],
    ) -> Result<String> {
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

    fn clean_wallet_transactions(&self) -> Result<json::CleanedWalletTransactions> {
        self.call("cleanwallettransactions", &[])
    }

    fn convert_passphrase(&self, passphrase: &str) -> Result<json::ConvertedPassphrase> {
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
        let mut args = [
            "*".into(),
            opt_into_json(minconf)?,
            opt_into_json(include_watchonly)?,
        ];
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
        addresses: Option<&[&Address]>,
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

    fn get_wallet_info(&self) -> Result<json::WalletInfo> {
        self.call("getwalletinfo", &[])
    }

    fn set_tx_fee(&self, amount: f64) -> Result<bool> {
        self.call("settxfee", &[amount.into()])
    }

    fn get_snapshot(&self, top: Option<String>) -> Result<Snapshot> {
        let mut args = [opt_into_json(top)?];
        self.call("getsnapshot", handle_defaults(&mut args, &[null()]))
    }

    // TOKENS

    fn tokenv2info(&self, token_id: &str) -> Result<TokenInfo> {
        self.call("tokenv2info", &[token_id.into()])
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{Auth, Client, ConfigFile};

    // todo https://github.com/iredelmeier/filesystem-rs/blob/master/src/lib.rs

    #[test]
    fn get_config() {
        let config_file = ConfigFile::new("VRSC").unwrap();
        println!("{:#?}", &config_file);

        let client = Client::chain("VRSC", Auth::ConfigFile);
        assert!(client.is_ok());

        let client = Client::chain(
            "VRSC",
            Auth::UserPass(
                "http://127.0.0.1:27777".to_string(),
                "1kj23k1l23".to_string(),
                "5jkhkjhl5".to_string(),
            ),
        );
        assert!(client.is_ok());
    }
}
