use std::{fs, path::Path};

use clap::{Parser, Subcommand};
use licenses_pro::{gen::AdminGenerator, LicenseStructParameters};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Config {
    count: i64,
    ivs: Vec<Vec<u8>>,
}
const CHUNK_SIZE: usize = 2;
fn main() {
    let args = Args::parse();
    match args.command {
        Command::NewConfig {
            file_path,
            payload_length,
            overwrite,
        } => {
            if Path::exists(Path::new(&file_path)) && !overwrite {
                eprintln!("File exists already. Run with --overwrite to overwrite file.");
                return;
            }
            let genner = AdminGenerator::new_with_random_ivs(LicenseStructParameters {
                seed_length: 64 / 8,
                payload_length,
                chunk_size: CHUNK_SIZE,
            });
            let config = Config {
                count: 0,
                ivs: genner.ivs,
            };
            let serialized = serde_json::to_vec(&config).expect("why would this even error lol");
            if let Err(err) = fs::write(file_path, serialized) {
                eprintln!("Error writing to file: {:?}", err);
                return;
            }
            eprintln!("Config created");
        }
        Command::NewLicense { path_to_config } => {
            let contents = match fs::read(path_to_config.clone()) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Error reading config file: {:?}", err);
                    return;
                }
            };
            let mut config: Config = match serde_json::from_slice(&contents) {
                Ok(parsed) => parsed,
                Err(err) => {
                    eprintln!("Error parsing config: {}", err);
                    return;
                }
            };

            let genner = AdminGenerator {
                parameters: LicenseStructParameters {
                    seed_length: 64 / 8,
                    payload_length: config.ivs.len(),
                    chunk_size: CHUNK_SIZE,
                },
                ivs: config.ivs.clone(),
            };
            let license=genner.generate_license(config.count.to_le_bytes().to_vec()).expect("Seed is set to be the amount of bytes in a 64 bit integer, so this should never fail");
            config.count += 1;
            if let Err(err) = fs::write(
                path_to_config,
                serde_json::to_vec(&config).expect("this shouldn't error"),
            ) {
                eprintln!(
                    "Error writing to file in order to increment count: {:?}",
                    err
                );
                return;
            }
            println!("{}", license.to_human_readable())
        }
    }
}
#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "Generate licenses for licenses_pro. The seed values here are generated based on a counter."
)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug, Clone)]
enum Command {
    #[clap(aliases=["conf","config"])]
    NewConfig {
        file_path: String,
        #[arg(default_value_t = 10)]
        payload_length: usize,
        #[arg(default_value_t=false,short,long)]
        overwrite: bool,
    },
    #[clap(aliases=["gen","new"])]
    NewLicense { path_to_config: String },
}
