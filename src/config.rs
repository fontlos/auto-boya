use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Parser)]
#[command(author, version, about = "BUAA Boya course automation tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Init the configuration
    Init,
    /// Show the configuration
    Show,
    /// Run(config is optional)
    Run(Setting),
    /// Set the configuration
    Set(Setting),
}

#[derive(Args, Deserialize, Serialize, Debug)]
pub struct Setting {
    /// Chrome binary path
    #[arg(long = "bin")]
    pub chrome_binary: Option<String>,
    /// Chrome driver path
    #[arg(long = "driver")]
    pub chrome_driver: Option<String>,
    /// Chrome driver port
    #[arg(long = "port")]
    pub driver_port: Option<String>,
    /// Your account
    #[arg(short, long)]
    pub account: Option<String>,
    /// Your password
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Tabled)]
pub struct Config {
    pub chrome_binary: String,
    pub chrome_driver: String,
    pub driver_port: String,
    pub account: String,
    pub password: String,
}

pub fn save_config(config: &Config) -> Result<()> {
    let cfg_str = toml::to_string(config)?;
    std::fs::write("./config.toml", cfg_str)?;
    Ok(())
}

pub fn init_config() -> Result<Config> {
    let chrome_binary: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Chrome binary path")
        .interact_text()?;

    let chrome_driver: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Chrome driver path")
        .interact_text()?;

    let driver_address: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Chrome driver address")
        .default("9515".to_string())
        .interact_text()?;

    let account: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your account")
        .interact_text()?;

    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your password")
        .interact_text()?;

    println!("{} {}", "[info]".green().bold(), "Init success");


    Ok(Config {
        chrome_binary,
        chrome_driver,
        driver_port: driver_address,
        account,
        password,
    })
}

pub fn get_config() -> Config {
    println!("{} {}", "[info]".green().bold(), "Read config");
    let cfg_str =match std::fs::read_to_string("./config.toml"){
        Ok(c)=> c,
        Err(_) => {
            println!("{} {}", "[warn]".yellow().bold(), "No config file found, try to initialize");
            let cfg = init_config().unwrap();
            save_config(&cfg).unwrap();
            return cfg;
        }
    };
    match toml::from_str::<Config>(&cfg_str){
        Ok(c)=> c,
        Err(_)=>{
            println!("{} {}", "[warn]".yellow().bold(), "Bad config, try to initialize");
            let cfg = init_config().unwrap();
            save_config(&cfg).unwrap();
            cfg
        }
    }
}

pub fn set_config(set: Setting) -> Config {
    let mut cfg = get_config();
    if let Some(s) = set.chrome_binary {
        cfg.chrome_binary = s;
    }
    if let Some(s) = set.chrome_driver {
        cfg.chrome_driver = s;
    }
    if let Some(s) = set.driver_port {
        cfg.driver_port = s;
    }
    if let Some(s) = set.account {
        cfg.account = s;
    }
    if let Some(s) = set.password {
        cfg.password = s;
    }
    println!("{} {}", "[info]".green().bold(), "Set success");
    cfg
}

#[test]
fn test_terminal() {
    use dialoguer::{theme::ColorfulTheme, Input};
    let _: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your email")
        .validate_with({
            let mut force = None;
            move |input: &String| -> Result<(), &str> {
                if input.contains('@') || force.as_ref().map_or(false, |old| old == input) {
                    Ok(())
                } else {
                    force = Some(input.clone());
                    Err("This is not a mail address; type the same value again to force use")
                }
            }
        })
        .interact_text()
        .unwrap();
}
