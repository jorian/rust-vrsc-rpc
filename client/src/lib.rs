#![crate_name = "vrsc_rpc"]
#![crate_type = "rlib"]

#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

pub extern crate jsonrpc;

pub extern crate vrsc_rpc_json;
pub use json::bitcoin;
pub use vrsc_rpc_json as json;
// pub use crate::client::coin_config::CoinConfig;

mod client;
mod error;
mod coin_config;

pub use client::*;
pub use error::Error;
