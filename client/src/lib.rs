#![crate_name = "vrsc_rpc"]
#![crate_type = "rlib"]

#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

mod chain_config;
pub mod client;
mod error;

pub extern crate jsonrpc;
pub extern crate vrsc_rpc_json;

pub use chain_config::*;
pub use error::Error;
pub use json::bitcoin;
pub use vrsc_rpc_json as json;
