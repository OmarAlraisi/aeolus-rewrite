use clap::Parser;
use serde::Deserialize;
use std::{error, fmt::Display, fs, io::BufReader};

const DEFAULT_NETWORK_INTERFACE: &str = "wlp1s0";
const DEFAULT_CONFIG_FILE: &str = "aeolus.yaml";

#[derive(Debug)]
pub struct ConfigError {
    description: String,
}

impl ConfigError {
    fn new(description: &str) -> Self {
        ConfigError {
            description: description.to_string(),
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Parser, Debug)]
struct CLIArgs {
    /// Path to Aeolus configuration file
    #[arg(long, value_name = "FILE", default_value_t = DEFAULT_CONFIG_FILE.to_string())]
    config: String,
}

#[derive(Deserialize)]
struct FileConfig {
    ports: Option<Vec<u16>>,
    servers: Vec<String>,
    iface: Option<String>,
}


#[derive(Debug)]
pub struct Config {
    pub ports: Vec<u16>,
    pub iface: String,
    pub servers: Vec<[u8; 6]>,
}

impl Config {
    pub fn parse() -> Result<Self, ConfigError> {
        let cli_args = CLIArgs::parse();
        let file_config = Config::parse_config_file(&cli_args.config)?;

        let servers = map_servers(file_config.servers.clone())?;
        let iface = file_config
            .iface
            .unwrap_or(DEFAULT_NETWORK_INTERFACE.to_string());

        Ok(Config {
            ports: file_config.ports.unwrap_or(vec![80]),
            iface,
            servers,
        })
    }

    fn parse_config_file(config_file: &str) -> Result<FileConfig, ConfigError> {
        let file_name = config_file.to_lowercase();
        if !(file_name.ends_with(".yaml") || file_name.ends_with(".yml")) {
            return Err(ConfigError::new("configuration file must be a YAML file."));
        }

        let file =
            fs::File::open(config_file).map_err(|e| ConfigError::new(e.to_string().as_str()))?;
        let reader = BufReader::new(file);
        let content: FileConfig = serde_yaml::from_reader(reader)
            .map_err(|e| ConfigError::new(e.to_string().as_str()))?;
        Ok(content)
    }
}

fn map_servers(raw_servers: Vec<String>) -> Result<Vec<[u8; 6]>, ConfigError> {
    let mut servers: Vec<[u8; 6]> = Vec::new();

    for server in raw_servers {
        let pairs: Vec<&str> = server.split(':').collect();
        if pairs.len() != 6 {
            return Err(ConfigError::new("Invalid MAC address."));
        }

        let mut bytes = [0; 6];
        for (idx, pair) in pairs.iter().enumerate() {
            bytes[idx] = u8::from_str_radix(pair, 16)
                .map_err(|_| ConfigError::new("Invalid MAC address."))?;
        }

        servers.push(bytes);
    }
    Ok(servers)
}
