use solana_program::system_instruction;
use solana_sdk::{
    hash::Hash, instruction::Instruction, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    signature::Signer, transaction::Transaction,
};

pub struct TxBuilder<'a, S> {
    fee_payer: Pubkey,
    instructions: Vec<Instruction>,
    signers: Vec<&'a S>,
}

impl<'a, S: Signer> TxBuilder<'a, S> {
    pub fn new(fee_payer: &'a S) -> Self {
        Self {
            fee_payer: fee_payer.pubkey(),
            instructions: vec![],
            signers: vec![fee_payer],
        }
    }

    pub fn build(&mut self, recent_blockhash: Hash) -> Transaction {
        let mut transaction =
            Transaction::new_with_payer(&self.instructions, Some(&self.fee_payer));
        transaction
            .try_sign(&self.signers, recent_blockhash)
            .expect("not enough signers");
        transaction
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn create_account(&mut self, account: &'a S, size: usize, owner: &Pubkey, rent: &Rent) {
        self.signers.push(account);
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
        account: &'a S,
        mint_authority: &Pubkey,
        mint_decimals: u8,
        rent: &Rent,
    ) {
        self.create_account(account, spl_token::state::Mint::LEN, &spl_token::id(), rent);
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

    // pub fn create_token_account(
    //     &mut self,
    //     account: &'a S,
    //     mint: &Pubkey,
    //     authority: &Pubkey,
    //     rent: &Rent,
    // ) {
    //     self.create_account(
    //         account,
    //         spl_token::state::Account::LEN,
    //         &spl_token::id(),
    //         rent,
    //     );
    //     self.add_instruction(
    //         spl_token::instruction::initialize_account2(
    //             &spl_token::id(),
    //             &account.pubkey(),
    //             mint,
    //             authority,
    //         )
    //         .unwrap(), // initialize_account2 never returns an Err
    //     );
    // }

    pub fn create_associated_token_account(&mut self, primary: &Pubkey, mint: &Pubkey) -> Pubkey {
        let account = spl_associated_token_account::get_associated_token_address(primary, mint);
        self.add_instruction(
            spl_associated_token_account::create_associated_token_account(
                &self.fee_payer,
                primary,
                mint,
            ),
        );
        account
    }

    pub fn mint_to(&mut self, mint: &Pubkey, authority: &'a S, account: &Pubkey, amount: u64) {
        self.signers.push(authority);
        self.add_instruction(
            spl_token::instruction::mint_to(
                &spl_token::id(),
                mint,
                account,
                &authority.pubkey(),
                &[&authority.pubkey()],
                amount,
            )
            .unwrap(),
        );
    }

    pub fn transfer(
        &mut self,
        source: &Pubkey,
        destination: &Pubkey,
        authority: &'a S,
        amount: u64,
    ) {
        self.signers.push(authority);
        self.add_instruction(
            spl_token::instruction::transfer(
                &spl_token::id(),
                source,
                destination,
                &authority.pubkey(),
                &[&authority.pubkey()],
                amount,
            )
            .unwrap(),
        );
    }
}
