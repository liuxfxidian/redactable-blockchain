use std::fs::File;
use std::io::Read;
use message::params::ThreasholdParam;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub threshold_params: ThreasholdParam,
    pub proxy_addr: String,
    pub node_addr: String,
    pub listen_addr: String
}

impl Config{
    pub fn load_config(path:&str)->String
    {
        let mut config_file = File::open(path).expect("Fail to open file!");
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).expect("Fail to read file contents");
        config_str
    }
}