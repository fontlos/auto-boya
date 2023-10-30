mod config;
mod run;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use tabled::{settings::Style, Table};
use thirtyfour::support::block_on;

use config::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let cfg = config::init_config().unwrap();
            config::save_config(&cfg).unwrap();
            println!("{} {}", "[info]".green().bold(), "Save success");
        }
        Command::Show => {
            let cfg = config::get_config();
            let mut t = Table::new(vec![cfg]);
            t.with(Style::rounded());
            println!("{t}");
        }
        Command::Run(set) => {
            block_on(run::run(set)).unwrap();
        }
        Command::Set(set) => {
            let cfg = config::set_config(set);
            config::save_config(&cfg).unwrap();
            println!("{} {}", "[info]".green().bold(), "Save success");
        }
    }
    Ok(())
}
