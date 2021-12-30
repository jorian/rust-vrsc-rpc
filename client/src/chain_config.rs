use crate::error::Error;
use os_info::Type as OSType;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, result};

pub type Result<T> = result::Result<T, Error>;

/// Let the system find a local installation, or supply your own connection details.
#[derive(Clone, Debug)]
pub enum Auth {
    UserPass(String, String, String),
    ConfigFile,
}

#[derive(Debug)]
pub struct ConfigFile {
    pub(crate) rpcuser: String,
    pub(crate) rpcpassword: String,
    pub(crate) rpcport: u16,
}

impl ConfigFile {
    fn get_komodo_installation_folder() -> Result<PathBuf> {
        if let Some(mut path) = dirs::home_dir() {
            match os_info::get().os_type() {
                OSType::Ubuntu | OSType::Linux => path.push(".komodo"),
                OSType::Macos | OSType::Windows => path.push("Komodo"),
                _ => return Err(Error::IOError(ErrorKind::Other.into())),
            }

            if !path.is_dir() {
                return Err(Error::IOError(ErrorKind::NotFound.into()));
            }

            Ok(path)
        } else {
            return Err(Error::IOError(ErrorKind::NotFound.into()));
        }
    }

    fn get_verustest_installation_folder() -> Result<PathBuf> {
        if let Some(mut path) = dirs::home_dir() {
            match os_info::get().os_type() {
                OSType::Ubuntu | OSType::Linux => path.push(".verustest"),
                OSType::Macos | OSType::Windows => path.push("VerusTest"),
                _ => return Err(Error::IOError(ErrorKind::Unsupported.into())),
            }

            path.push("pbaas");

            if !path.is_dir() {
                return Err(Error::IOError(ErrorKind::NotADirectory.into()));
            }

            Ok(path)
        } else {
            return Err(Error::IOError(ErrorKind::NotFound.into()));
        }
    }

    pub fn new(name: &str) -> Result<Self> {
        let mut path;
        match name {
            v if v.to_ascii_uppercase() == "VRSC" => {
                path = self::ConfigFile::get_komodo_installation_folder()?;
                path.push(v);
                path.push(&format!("{}.conf", v));
            }
            vt if vt.to_ascii_lowercase() == "vrsctest" => {
                path = self::ConfigFile::get_komodo_installation_folder()?;
                path.push(vt);
                path.push(&format!("{}.conf", vt));
            }
            _x => {
                path = self::ConfigFile::get_verustest_installation_folder()?;
                path.push(_x.to_ascii_lowercase());
                path.push(format!("{}.conf", _x.to_ascii_lowercase()));
            }
        }

        if !path.exists() {
            return Err(Error::IOError(ErrorKind::NotFound.into()));
        }

        let contents = fs::read_to_string(path.to_str().unwrap())?;

        let map: HashMap<String, String> = contents
            .as_str()
            .split('\n')
            .map(|line| line.splitn(2, '=').collect::<Vec<&str>>())
            .filter(|vec| vec.len() == 2)
            .map(|vec| (vec[0].to_string(), vec[1].to_string()))
            .collect::<HashMap<String, String>>();

        let rpc_user = map.get("rpcuser").ok_or(Error::InvalidConfigFile)?;
        let rpc_password = map.get("rpcpassword").ok_or(Error::InvalidConfigFile)?;
        let rpc_port = match name {
            // VRSC doesn't put rpcport in conf file at install, but users could have modified it afterwards.
            "VRSC" => match map.get("rpcport") {
                Some(port) => port,
                None => "8232",
            },
            _ => map.get("rpcport").ok_or(Error::InvalidConfigFile)?,
        };

        Ok(ConfigFile {
            rpcuser: rpc_user.to_owned(),
            rpcpassword: rpc_password.to_owned(),
            rpcport: rpc_port.parse::<u16>()?,
        })
    }
}
