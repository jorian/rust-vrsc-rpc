use crate::error::Error;
use os_info::Type as OSType;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{fs, result};
use tracing::debug;

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
    pub fn get_komodo_installation_folder() -> Result<PathBuf> {
        let mut full_path = PathBuf::new();
        match os_info::get().os_type() {
            OSType::Ubuntu | OSType::Linux | OSType::Debian | OSType::OracleLinux => {
                if let Some(path) = dirs::home_dir() {
                    full_path.push(path);
                    full_path.push(".komodo");
                } else {
                    return Err(Error::IOError(ErrorKind::NotFound.into()));
                }
            }
            OSType::Macos | OSType::Windows => {
                if let Some(path) = dirs::data_local_dir() {
                    full_path.push(path);
                    full_path.push("Komodo")
                } else {
                    return Err(Error::IOError(ErrorKind::NotFound.into()));
                }
            }
            _ => return Err(Error::IOError(ErrorKind::Other.into())),
        }

        if !full_path.is_dir() {
            return Err(Error::IOError(ErrorKind::NotFound.into()));
        }

        Ok(full_path)
    }

    pub fn get_verustest_installation_folder() -> Result<PathBuf> {
        if let Some(mut path) = dirs::home_dir() {
            match os_info::get().os_type() {
                OSType::Ubuntu | OSType::Linux | OSType::Debian => path.push(".verustest"),
                OSType::Macos => {
                    path.push("Library");
                    path.push("Application Support");
                    path.push("VerusTest");
                }
                _ => return Err(Error::IOError(ErrorKind::Unsupported.into())),
            }

            debug!("{:?}", &path);

            path.push("pbaas");

            if !path.is_dir() {
                return Err(Error::IOError(ErrorKind::NotFound.into()));
            }

            Ok(path)
        } else {
            return Err(Error::IOError(ErrorKind::NotFound.into()));
        }
    }

    pub fn pbaas(testnet: bool, currencyidhex: &str) -> Result<Self> {
        let mut path;

        match testnet {
            true => {
                path = self::ConfigFile::get_verustest_installation_folder()?;
                path.push(currencyidhex);
                path.push(format!("{}.conf", currencyidhex));

                debug!("{:?}", &path);
                get_config(&path)
            }
            false => unimplemented!(),
        }
    }

    pub fn vrsc(testnet: bool) -> Result<Self> {
        let mut path;
        match testnet {
            true => {
                path = self::ConfigFile::get_komodo_installation_folder()?;
                path.push("vrsctest");
                path.push("vrsctest.conf");
            }
            false => {
                path = self::ConfigFile::get_komodo_installation_folder()?;
                path.push("VRSC");
                path.push("VRSC.conf");
            }
        }

        debug!("{:?}", &path);
        get_config(&path)
    }
}

pub fn get_config(path: &Path) -> Result<ConfigFile> {
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
    let rpc_port = match path.ends_with("VRSC.conf") {
        // VRSC doesn't put rpcport in conf file at install, but users could have modified it afterwards.
        true => match map.get("rpcport") {
            Some(port) => port,
            None => "8232",
        },
        false => map.get("rpcport").ok_or(Error::InvalidConfigFile)?,
    };

    Ok(ConfigFile {
        rpcuser: rpc_user.to_owned(),
        rpcpassword: rpc_password.to_owned(),
        rpcport: rpc_port.parse::<u16>()?,
    })
}
