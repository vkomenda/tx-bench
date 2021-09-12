use crate::tx_builder::TxBuilder;
use log::{error, info};
use solana_client::{client_error::ClientError, rpc_client::RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, rent::Rent, signer::Signer, sysvar::rent};

pub trait RpcActions {
    fn execute_tx<'a, S>(&self, tx_builder: &mut TxBuilder<'a, S>) -> Result<(), ClientError>
    where
        S: Signer;
    fn rent(&self) -> Result<Rent, ClientError>;
}

impl RpcActions for RpcClient {
    fn execute_tx<'a, S>(&self, tx_builder: &mut TxBuilder<'a, S>) -> Result<(), ClientError>
    where
        S: Signer,
    {
        match self.send_and_confirm_transaction_with_spinner_and_commitment(
            &tx_builder.build(self.get_recent_blockhash()?.0),
            CommitmentConfig::confirmed(),
        ) {
            Ok(signature) => {
                info!("Signature {}", signature);
            }
            Err(e) => {
                error!("{}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    fn rent(&self) -> Result<Rent, ClientError> {
        let rent = bincode::deserialize(&self.get_account_data(&rent::id())?)
            .expect("cannot deserialize rent");
        Ok(rent)
    }
}
