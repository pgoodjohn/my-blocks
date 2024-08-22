use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::borrow::Borrow;
use std::fs::File;
use std::path::PathBuf;
use toml;
use plogger;

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub version: String,
    #[serde(rename = "developmentMode")]
    pub development_mode: bool,
    #[serde(rename = "configurationPath")]
    pub config_path: PathBuf,
    #[serde(rename = "dbPath")]
    pub db_path: PathBuf,
    #[serde(rename="workspaceId")]
    pub workspace_id: Uuid
}

impl Configuration {
    fn config_path(dev_mode: bool) -> PathBuf {

        if dev_mode {
            let mut config_path = PathBuf::new();
            config_path.push(".config.toml");

            return config_path;
        }

        let mut config_path = PathBuf::new();
        config_path.push(dirs::home_dir().expect("Could not load home dir"));
        config_path.push(".config/.my-blocks/config.toml");

        println!("Loading config_path {:?}", config_path);

        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                log::info!("Creating configuration directory for {:?}", &config_path);
                std::fs::create_dir_all(parent).expect("Could not create configuration directory");
                println!("Directory created: {:?}", parent);
            }
        }

        if !config_path.exists() {
            log::info!("Creating configuration file {:?}", &config_path);
            File::create(&config_path).expect("Could not create config file");
        }

        config_path
    }

    fn db_path(dev_mode: bool) -> PathBuf {

        if dev_mode {
            let mut config_path = PathBuf::new();
            config_path.push("file.db");

            return config_path;
        }

        let mut db_path = PathBuf::new();
        db_path.push(dirs::home_dir().expect("Could not load home dir"));
        db_path.push(".config/.my-blocks/db.sqlite");

        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).expect("Could not create configuration directory");
                println!("Directory created: {:?}", parent);
            }
        }

        if !db_path.exists() {
            File::create(&db_path).expect("Could not create config file");
        }

        db_path
    }

    fn load_from_file(dev_mode: bool) -> Result<Self, String> {
        let config_path = Configuration::config_path(dev_mode);
        log::debug!("Loading config from {:?}", &config_path);
        let config_str = std::fs::read_to_string(&config_path);

        match config_str {
            Ok(config_str) => {
                log::debug!("Configuration successfully loaded from file");
                match toml::from_str(&config_str) {
                    Ok(config) => Ok(config),
                    Err(e) => {
                        log::error!("Could not parse config file: {:?}", e);
                        Err(String::from("Could not parse config file"))
                    }
                }
            },
            Err(_e) => {
                log::debug!("Configuration file not found, bootstrapping new configuration");
                Ok(Configuration::bootstrap(dev_mode).unwrap())
            }
        }
    }

    fn bootstrap (dev_mode: bool) -> Result<Self, String> {

        let config = Configuration {
            version: String::from(env!("CARGO_PKG_VERSION")),
            development_mode: dev_mode,
            config_path: Configuration::config_path(dev_mode),
            db_path: Configuration::db_path(dev_mode),
            workspace_id: Uuid::now_v7()
        };

        config.save().expect("Could not save config file");

        Ok(config)
    }

    fn save(&self) -> Result<(), String> {
        let config_path = PathBuf::from(&self.config_path);
        let config_str = toml::to_string(&self).expect("Could not serialize config");

        match std::fs::write(&config_path, config_str) {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Could not write config file: {:?}", e);
                Err(String::from("Could not write config file"))
            }
        }
    }

    pub fn init() -> Result<Self, String> {
        let dev_mode = cfg!(debug_assertions);
        plogger::init(dev_mode);
        log::debug!("Logger initialised");
        log::debug!("Initializing configuration with dev mode - {:?}", dev_mode);

        let config = Configuration::load_from_file(dev_mode).expect("Could not load configuration");

        log::debug!("Configuration initialised - {:?}", config);

        Ok(config)
    }
}

#[tauri::command]
pub fn load_configuration_command(configuration: State<Configuration>) -> String {
    log::debug!("Running load_configuration_command. {:?}", configuration);

    serde_json::to_string(&configuration.inner()).unwrap()
}
