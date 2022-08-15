use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

const CONFIG_PATH: &str = ".escrow_cli_config";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub signer: String,
}

pub fn get_config() -> Option<Config> {
    let path = Path::new(CONFIG_PATH);
    if !path.exists() {
        None
    } else {
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read config file");
        let config = serde_json::from_str(&contents).unwrap();
        Some(config)
    }
}

pub fn update_signer(signer: &str) -> Result<(), String> {
    let config = Config {
        signer: signer.to_string(),
    };
    write_config(&config)?;
    Ok(())
}

pub fn write_config(config: &Config) -> Result<(), String> {
    let serialized = serde_json::to_string(config).unwrap();
    let mut file = File::create(CONFIG_PATH).unwrap();
    file.write_all(&serialized.into_bytes()).unwrap();
    Ok(())
}
