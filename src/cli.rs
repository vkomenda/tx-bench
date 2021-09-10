use clap::{crate_description, crate_name, App, Arg, ArgMatches};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};
use std::str::FromStr;

const TOKEN_PROGRAM_ID: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub struct Config {
    pub id: Keypair,
    pub num_keypairs: u64,
    pub token_program_id: Pubkey,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            id: Keypair::new(),
            num_keypairs: 400,
            token_program_id: Pubkey::from_str(TOKEN_PROGRAM_ID)
                .expect("cannot parse default token program ID"),
        }
    }
}

pub fn build_args<'a, 'b>(version: &'b str) -> App<'a, 'b> {
    App::new(crate_name!())
        .about(crate_description!())
        .version(version)
        .arg(Arg::with_name("identity").short("i").takes_value(true))
        .arg(Arg::with_name("num_keypairs").short("n").takes_value(true))
        .arg(
            Arg::with_name("token_program_id")
                .short("t")
                .takes_value(true),
        )
}

pub fn extract_args(matches: &ArgMatches) -> Config {
    let mut args = Config::default();

    if let Some(k) = matches.value_of("identity") {
        args.id = read_keypair_file(k).expect("can't read client identity");
    }

    if let Some(n) = matches.value_of("num_keypairs") {
        args.num_keypairs = n.to_string().parse().expect("can't parse num_keypairs");
    }

    if let Some(s) = matches.value_of("token_program_id") {
        args.token_program_id = Pubkey::from_str(s).expect("can't read token program pubkey");
    }

    args
}
