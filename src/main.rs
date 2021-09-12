mod cli;
mod rpc_actions;
mod tx_builder;

use crate::cli::Config;
use anyhow::anyhow;
use log::{info, LevelFilter};
use rpc_actions::RpcActions;
use solana_client::{client_error::ClientError, rpc_client::RpcClient};
use solana_core::gen_keys::GenKeys;
use solana_sdk::{hash::Hash, signature::Keypair, signer::Signer};
use statistical::{mean, median, standard_deviation, variance};
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn copy_seed(keypair: &Keypair) -> [u8; 32] {
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&keypair.to_bytes()[..32]);
    seed
}

#[inline]
fn add_entropy(seed: &mut [u8; 32], entropy: &[u8; 32]) {
    for (seed_byte, entropy_byte) in seed.iter_mut().zip(entropy.iter()) {
        *seed_byte ^= *entropy_byte;
    }
}

fn generate_keypair(seed_keypair: &Keypair, recent_blockhash: Hash) -> Keypair {
    let mut seed = copy_seed(seed_keypair);
    add_entropy(&mut seed, &recent_blockhash.0);
    let mut rnd = GenKeys::new(seed);
    rnd.gen_keypair()
}

fn generate_keypairs(seed_keypair: &Keypair, recent_blockhash: Hash, count: u64) -> Vec<Keypair> {
    let mut seed = copy_seed(seed_keypair);
    add_entropy(&mut seed, &recent_blockhash.0);
    let mut rnd = GenKeys::new(seed);
    rnd.gen_n_keypairs(count)
}

fn create_mint_account(client: &RpcClient, config: &Config) -> Result<Keypair, ClientError> {
    let recent_blockhash = client.get_recent_blockhash()?.0;
    let mint = generate_keypair(&config.id, recent_blockhash);
    info!("Mint pubkey {}", mint.pubkey());
    let mut tx_builder = tx_builder::TxBuilder::new(&config.id);
    tx_builder.create_mint_account(&mint, &config.id.pubkey(), 6, &client.rent()?);
    client.execute_tx(&mut tx_builder)?;
    Ok(mint)
}

fn with_duration<F, A>(mut f: F) -> anyhow::Result<(A, Duration)>
where
    F: FnMut() -> anyhow::Result<A>,
{
    let start = Instant::now();
    let a = f()?;
    let duration = start.elapsed();
    Ok((a, duration))
}

fn log_duration_stats<'a, I>(durations: I, message: &str)
where
    I: Iterator<Item = &'a Duration>,
{
    let secs: Vec<f64> = durations.map(|d| d.as_secs_f64()).collect();
    info!(
        "{}: min {:.6}s, max {:.6}s, mean {:.6}s, median {:.6}s, standard deviation {:.6}, variance {:.6}",
        message,
        secs.iter().cloned().reduce(|x, y| x.min(y)).unwrap(),
        secs.iter().cloned().reduce(|x, y| x.max(y)).unwrap(),
        mean(&secs),
        median(&secs),
        standard_deviation(&secs, None),
        variance(&secs, None),
    );
}

fn main() -> anyhow::Result<()> {
    env_logger::builder().filter(None, LevelFilter::Info).init();

    let matches = cli::build_args(VERSION).get_matches();
    let config = cli::extract_args(&matches);
    let client = RpcClient::new(config.url.clone());
    let (mint, mint_duration) =
        with_duration(|| create_mint_account(&client, &config).map_err(|e| anyhow!("{}", e)))?;
    info!(
        "Mint account {} created in {:?}",
        mint.pubkey(),
        mint_duration
    );
    let keypairs = generate_keypairs(
        &config.id,
        client.get_recent_blockhash()?.0,
        config.num_keypairs,
    );

    let mut token_account_durations = vec![];
    let mut token_accounts = vec![];
    for keypair in keypairs {
        let mut tx_builder = tx_builder::TxBuilder::new(&config.id);
        let token_account =
            tx_builder.create_associated_token_account(&keypair.pubkey(), &mint.pubkey());
        token_accounts.push(token_account);
        let token_account_duration = with_duration(|| {
            client
                .execute_tx(&mut tx_builder)
                .map_err(|e| anyhow!("{}", e))
        })?
        .1;
        info!(
            "Token account {} created in {:?}",
            token_account, token_account_duration,
        );
        token_account_durations.push(token_account_duration);
    }
    log_duration_stats(
        token_account_durations.iter(),
        "create_associated_token_account",
    );

    // Mint tokens to the account of the sender.
    let mut tx_builder = tx_builder::TxBuilder::new(&config.id);
    let id_token_account =
        tx_builder.create_associated_token_account(&config.id.pubkey(), &mint.pubkey());
    tx_builder.mint_to(
        &mint.pubkey(),
        &config.id,
        &id_token_account,
        config.num_keypairs,
    );
    let mint_to_duration = with_duration(|| {
        client
            .execute_tx(&mut tx_builder)
            .map_err(|e| anyhow!("{}", e))
    })?
    .1;
    info!(
        "Source token account {} created tokens were minted to it in {:?}",
        id_token_account, mint_to_duration,
    );

    let mut transfer_durations = vec![];
    for token_account in token_accounts {
        let mut tx_builder = tx_builder::TxBuilder::new(&config.id);
        tx_builder.transfer(&id_token_account, &token_account, &config.id, 1);
        let transfer_duration = with_duration(|| {
            client
                .execute_tx(&mut tx_builder)
                .map_err(|e| anyhow!("{}", e))
        })?
        .1;
        info!(
            "Transfer to {:?} done in {:?}",
            token_account, transfer_duration,
        );
        transfer_durations.push(transfer_duration);
    }
    log_duration_stats(transfer_durations.iter(), "transfer");

    Ok(())
}
