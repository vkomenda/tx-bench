use solana_program::system_instruction;
use solana_sdk::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey, rent::Rent, signature::Signer,
    transaction::Transaction,
};

pub struct TxBuilder<S> {
    fee_payer: Pubkey,
    instructions: Vec<Instruction>,
    signers: Vec<S>,
}

impl<S: Clone + Signer> TxBuilder<S> {
    pub fn new(fee_payer: S) -> Self {
        Self {
            fee_payer: fee_payer.pubkey(),
            instructions: vec![],
            signers: vec![fee_payer],
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn create_account(&mut self, account: S, size: usize, owner: &Pubkey, rent: &Rent) {
        self.signers.push(account.clone());
        self.add_instruction(system_instruction::create_account(
            &self.fee_payer,
            &account.pubkey(),
            rent.minimum_balance(size),
            size as u64,
            owner,
        ));
    }

    pub fn create_mint_account(
        &mut self,
        account: S,
        mint_authority: &Pubkey,
        mint_decimals: u8,
        rent: &Rent,
    ) {
        self.create_account(
            account.clone(),
            spl_token::state::Mint::LEN,
            &spl_token::id(),
            rent,
        );
        self.add_instruction(
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &account.pubkey(),
                mint_authority,
                None,
                mint_decimals,
            )
            .expect("cannot create initialize_mint"),
        );
    }
}
