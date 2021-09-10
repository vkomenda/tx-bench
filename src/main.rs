use solana_core::gen_keys::GenKeys;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::iter;
//use std::{any::Any, collections::HashSet};

mod cli;
mod tx_builder;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn generate_keypairs(seed_keypair: &Keypair, count: u64) -> Vec<Keypair> {
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_keypair.to_bytes()[..32]);
    let mut rnd = GenKeys::new(seed);
    rnd.gen_n_keypairs(count)
}

fn main() {
    let matches = cli::build_args(VERSION).get_matches();
    let config = cli::extract_args(&matches);
    let keypairs = generate_keypairs(&config.id, config.num_keypairs);
}
