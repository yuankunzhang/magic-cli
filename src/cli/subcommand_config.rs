use std::error::Error;

use clap::Subcommand;
use colored::Colorize;
use inquire::Text;

use super::config::{ConfigKeys, MagicCliConfig};

#[derive(Subcommand)]
pub enum ConfigSubcommands {
    /// Set a value.
    Set {
        /// The key to set in the configuration. If not provided, you will be prompted to select one.
        #[arg(short, long)]
        key: Option<String>,
        /// The value to set. If not provided, you will be prompted to enter a value.
        #[arg(short, long)]
        value: Option<String>,
    },
    /// Get a value.
    Get {
        /// The key to get from the configuration. If not provided, you will be prompted to select one.
        #[arg()]
        key: Option<String>,
    },
    /// List the configurations.
    List,
    /// Reset the configurations to the default values.
    Reset,
    /// Get the path to the configuration file.
    Path,
}

pub struct ConfigSubcommand;

impl ConfigSubcommand {
    pub fn run(command: &ConfigSubcommands) -> Result<(), Box<dyn Error>> {
        match command {
            ConfigSubcommands::Set { key, value } => {
                let key = match key {
                    Some(key) => key.to_string(),
                    None => MagicCliConfig::select_key()?,
                };
                let value = match value {
                    Some(value) => value.to_string(),
                    // TODO: Support secrets.
                    None => Text::new(&format!("{} {}: ", "Enter the value for the key", key.magenta())).prompt()?,
                };

                match MagicCliConfig::set(&key, &value) {
                    Ok(_) => println!("{}", "Configuration updated.".green().bold()),
                    Err(err) => {
                        eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                        return Err(Box::new(err));
                    }
                }
            }
            ConfigSubcommands::Get { key } => {
                let key = match key {
                    Some(key) => key.to_string(),
                    None => MagicCliConfig::select_key()?,
                };
                match MagicCliConfig::get(&key) {
                    Ok(value) => println!("{}", value),
                    Err(err) => {
                        eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                        return Err(Box::new(err));
                    }
                }
            }
            ConfigSubcommands::List => {
                let config_keys = ConfigKeys::keys();
                let config_keys = config_keys.get().unwrap();
                let mut sorted_config_keys = config_keys.values().collect::<Vec<_>>();
                sorted_config_keys.sort_by(|a, b| a.prio.cmp(&b.prio).then(a.key.cmp(&b.key)));
                for (i, item) in sorted_config_keys.iter().enumerate() {
                    let config_value = MagicCliConfig::get(&item.key)?;
                    let config_value = config_value.replace("null", "-");
                    let config_value = if item.is_secret {
                        "*".repeat(config_value.len())
                    } else {
                        config_value
                    };
                    println!(
                        "Field: {} {}\nValue: {}\nDescription: {}",
                        item.key.blue().bold(),
                        if item.is_secret { "(secret)".yellow() } else { "".dimmed() },
                        config_value.bold(),
                        item.description.dimmed(),
                    );
                    if i < config_keys.len() - 1 {
                        println!();
                    }
                }
            }

            ConfigSubcommands::Reset => {
                MagicCliConfig::reset()?;
                println!("{}", "Configuration reset to default values.".green().bold());
            }
            ConfigSubcommands::Path => {
                let config = MagicCliConfig::get_config_file_path()?;
                println!("{}", config.display());
            }
        }
        Ok(())
    }
}