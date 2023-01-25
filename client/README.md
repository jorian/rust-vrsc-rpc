# Client implementations

The client side of making RPC requests to the Verus daemon.

This crate handles the definition of the Remote-Prodecure-Calls as is defined by the Verus Daemon. It tries to be a stateless wrapper of the RPCs.

## Notes

- Accounts are not supported in Verus and will not be implemented here.

### Addressindex

- [ ] getaddressbalance
- [ ] getaddressdeltas
- [ ] getaddressmempool
- [ ] getaddresstxids
- [ ] getaddressutxos
- [ ] getsnapshot

### Blockchain

<!-- - [x] getsnapshot ( "top" ) -->

- [x] coinsupply < height >
- [x] getbestblockhash
- [x] getblock "hash|height" ( verbosity )
- [x] getblockchaininfo
- [x] getblockcount
- [ ] getblockdeltas "blockhash"
- [x] getblockhash index
- [ ] getblockhashes timestamp
- [x] getblockheader "hash" ( verbose )
- [x] getchaintips
- [x] getchaintxstats
- [x] getdifficulty
<!-- - [ ] getlastsegidstakes depth -->
- [x] getmempoolinfo
- [x] getrawmempool ( verbose )
- [ ] getspentinfo
- [x] gettxout "txid" n ( includemempool )
- [x] gettxoutproof [ "txid",... ] ( blockhash )
- [x] gettxoutsetinfo
- [ ] ~~kvsearch key~~
- [ ] ~~kvupdate key "value" days passphrase~~
- [x] minerids height
- [x] notaries height timestamp
- [x] verifychain ( checklevel numblocks )
- [x] verifytxoutproof "proof"
- [ ] z_gettreestate "hash|height"

### Control

- [ ] getinfo
- [ ] help ( "command" )
- [ ] stop

### Crosschain

- [ ] MoMoMdata symbol kmdheight ccid
- [ ] assetchainproof needs a txid
- [ ] calc_MoM height MoMdepth
- [ ] getNotarisationsForBlock blockHash
- [ ] height_MoM height
- [ ] migrate_completeimporttransaction importTx
- [ ] migrate_converttoexport rawTx dest_symbol export_amount
- [ ] migrate_createimporttransaction burnTx payouts
- [ ] scanNotarisationsDB blockHeight symbol [ blocksLimit=1440 ]

### Disclosure

z_getpaymentdisclosure "txid" "js_index" "output_index" ("message")
z_validatepaymentdisclosure "paymentdisclosure"

### Generating

- [ ] generate numblocks
- [ ] getgenerate
- [ ] setgenerate generate ( genproclimit )

### Identity

- [x] getidentity "name@ || iid" (height) (txproof) (txproofheight)
- [x] listidentities (includecansign) (includewatchonly)
- [ ] recoveridentity "jsonidentity" (returntx)
- [ ] registeridentity "jsonidregistration" (returntx) feeoffer
- [x] registernamecommitment "name" "controladdress" ("referralidentity")
- [ ] revokeidentity "nameorID" (returntx)
- [ ] setidentitytimelock "id@" '{"unlockatblock":absoluteblockheight || "setunlockdelay":numberofblocksdelayafterunlock}' (returntx)
- [ ] signfile "address or identity" "filepath/filename" "curentsig"
- [ ] signmessage "address or identity" "message" "currentsig"
- [ ] updateidentity "jsonidentity" (returntx)
- [ ] verifyfile "address or identity" "signature" "filepath/filename" "checklatest"
- [ ] verifyhash "address or identity" "signature" "hexhash" "checklatest"
- [ ] verifymessage "address or identity" "signature" "message" "checklatest"

### Marketplace

- [ ] closeoffers ('["offer1_txid", "offer2_txid", ...]') (transparentorprivatefundsdestination) (privatefundsdestination)
- [ ] getoffers "currencyorid" (iscurrency) (withtx)
- [ ] listopenoffers (unexpired) (expired)'
- [ ] makeoffer fromaddress '{"changeaddress":"transparentoriaddress", "expiryheight":blockheight, "offer":{"currency":"anycurrency", "amount":...} | {"identity":"idnameoriaddress",...}', "for":{"address":..., "currency":"anycurrency", "amount":...} | {"name":"identityforswap","parent":"parentid","primaryaddresses":["R-address(s)"],"minimumsignatures":1,...}}' (returntx) (feeamount)
- [ ] takeoffer fromaddress '{"txid":"txid" | "tx":"hextx", "changeaddress":"transparentoriaddress", "deliver":"fullidnameoriaddresstodeliver" | {"currency":"currencynameorid","amount":n}, "accept":{"address":"addressorid","currency":"currencynameorid","amount"} | {identitydefinition}}' (returntx) (feeamount)

### Mining

<!-- - [ ] genminingCSV -->

- [ ] getblocksubsidy height
- [ ] getblocktemplate ( "jsonrequestobject" )
- [ ] getlocalsolps
- [ ] getmininginfo
- [ ] getnetworkhashps ( blocks height )
- [ ] getnetworksolps ( blocks height )
- [ ] prioritisetransaction <txid> <priority delta> <fee delta>
- [ ] submitblock "hexdata" ( "jsonparametersobject" )

### Network

- [ ] addnode "node" "add|remove|onetry"
- [ ] clearbanned
- [ ] disconnectnode "node"
- [ ] getaddednodeinfo dns ( "node" )
- [ ] getconnectioncount
- [ ] getdeprecationinfo
- [ ] getnettotals
- [ ] getnetworkinfo
- [x] getpeerinfo
- [ ] listbanned
- [ ] ping
- [ ] setban "ip(/netmask)" "add|remove" (bantime) (absolute)

### Rawtransactions

- [ ] createrawtransaction [{"txid":"id","vout":n},...] {"address":amount,...} ( locktime ) ( expiryheight )
- [ ] decoderawtransaction "hexstring"
- [ ] decodescript "hex"
- [ ] fundrawtransaction "hexstring"
- [x] getrawtransaction "txid" ( verbose )
- [ ] sendrawtransaction "hexstring" ( allowhighfees )
- [ ] signrawtransaction "hexstring" ( [{"txid":"id","vout":n,"scriptPubKey":"hex","redeemScript":"hex"},...] ["privatekey1",...] sighashtype )

### Util

- [ ] createmultisig nrequired ["key",...]
<!-- - [ ] decodeccopret scriptPubKey -->
- [ ] estimatefee nblocks
- [ ] estimatepriority nblocks
- [ ] invalidateblock "hash"
- [ ] jumblr_deposit "depositaddress"
- [ ] jumblr_pause
- [ ] jumblr_resume
- [ ] jumblr_secret "secretaddress"
- [ ] reconsiderblock "hash"
<!-- - [ ] txnotarizedconfirmed txid -->
- [ ] validateaddress "komodoaddress"
<!-- - [ ] verifymessage "komodoaddress" "signature" "message" -->
- [ ] z_validateaddress "zaddr"

### VDXF

- [ ] getvdxfid "vdxfuri"

### Wallet

- [x] addmultisigaddress nrequired ["key",...] ( "account" )
- [x] backupwallet "destination"
<!-- - [x] cleanwallettransactions "txid" -->
- [x] convertpassphrase "agamapassphrase"
- [x] dumpprivkey "t-addr"
- [ ] dumpwallet "filename"
- [ ] encryptwallet "passphrase"
- [x] ~~getaccount "VRSC_address"~~
- [x] ~~getaccountaddress "account"~~
- [x] ~~getaddressesbyaccount "account"~~
- [x] getbalance ( ~~"account"~~ minconf includeWatchonly )
- [ ] getcurrencybalance "address" ( minconf friendlynames)
- [x] getnewaddress ~~( "account" )~~
- [x] getrawchangeaddress
- [x] ~~getreceivedbyaccount "account" ( minconf )~~
- [x] getreceivedbyaddress "VRSC_address" ( minconf )
- [x] gettransaction "txid" ( includeWatchonly )
- [x] getunconfirmedbalance
- [x] getwalletinfo
- [x] importaddress "address" ( "label" rescan )
- [x] importprivkey "komodoprivkey" ( "label" rescan height secret_key)
- [ ] importwallet "filename"
- [x] keypoolrefill ( newsize )
- [x] ~~listaccounts ( minconf includeWatchonly)~~
- [x] ~~listaddressgroupings~~
- [x] listlockunspent
- [x] ~~listreceivedbyaccount ( minconf includeempty includeWatchonly)~~
- [x] listreceivedbyaddress ( minconf includeempty includeWatchonly)
- [x] listsinceblock ( "blockhash" target-confirmations includeWatchonly)
- [x] listtransactions ( "account" count from includeWatchonly)
- [x] listunspent ( minconf maxconf ["address",...] )
- [x] lockunspent unlock [{"txid":"txid","vout":n},...]
- [ ] ~~move "fromaccount" "toaccount" amount ( minconf "comment" )~~
<!-- - [x] opreturn_burn burn_amount hexstring ( txfee ) -->
- [x] resendwallettransactions
- [ ] ~~sendfrom "fromaccount" "toKMDaddress" amount ( minconf "comment" "comment-to" )~~
- [x] sendmany ~~"fromaccount"~~ {"address":amount,...} ( minconf "comment" ["address",...] )
- [x] sendtoaddress "KMD_address" amount ( "comment" "comment-to" subtractfeefromamount )
- [x] ~~setaccount "KMD_address" "account"~~
  <!-- - [ ] ~~setpubkey~~ invalid response -->
  <!-- - [ ] setstakingsplit -->
- [x] settxfee amount
  <!-- - [x] signmessage "t-addr" "message" -->
  <!-- - [ ] walletlock -->
  <!-- - [ ] walletpassphrase "passphrase" timeout -->
  <!-- - [ ] walletpassphrasechange "oldpassphrase" "newpassphrase" -->
- [ ] z_exportkey "zaddr"
- [ ] z_exportviewingkey "zaddr"
- [ ] z_exportwallet "filename"
- [ ] z_getbalance "address" ( minconf )
- [ ] z_getnewaddress ( type )
- [ ] z_getoperationresult (["operationid", ... ])
- [ ] z_getoperationstatus (["operationid", ... ])
- [ ] z_gettotalbalance ( minconf includeWatchonly )
- [ ] z_importkey "zkey" ( rescan startHeight )
- [ ] z_importviewingkey "vkey" ( rescan startHeight )
- [ ] z_importwallet "filename"
- [ ] z_listaddresses ( includeWatchonly )
- [ ] z_listoperationids
- [ ] z_listreceivedbyaddress "address" ( minconf )
- [ ] z_listunspent ( minconf maxconf includeWatchonly ["zaddr",...] )
- [ ] z_mergetoaddress ["fromaddress", ... ] "toaddress" ( fee ) ( transparent_limit ) ( shielded_limit ) ( memo )
- [ ] z_sendmany "fromaddress" [{"address":... ,"amount":...},...] ( minconf ) ( fee )
- [ ] z_shieldcoinbase "fromaddress" "tozaddress" ( fee ) ( limit )
- [ ] z_viewtransaction "txid"
- [ ] zcbenchmark benchmarktype samplecount
- [ ] zcrawjoinsplit rawtx inputs outputs vpub_old vpub_new
- [ ] zcrawkeygen
- [ ] zcrawreceive zcsecretkey encryptednote
- [ ] zcsamplejoinsplit
