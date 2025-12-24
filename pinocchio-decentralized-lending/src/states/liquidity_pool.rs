use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidityPool {
    pub authority: Pubkey, //Who controls the pool
    pub mint_a: Pubkey, //Liquidity pool token 1
    pub mint_b: Pubkey, //Liquidity pool token 2
    pub lp_mint: Pubkey, //Lp token mint
    pub vault_a: Pubkey, //Vault to hold mint_a tokens
    pub vault_b: Pubkey, //Vault to hold mint_b tokens
    pub fees_vault_a: Pubkey,
    pub fees_vault_b: Pubkey,
    pub total_liquidity: u64, //Whats the total liquidity provided 
    pub total_borrowed_a: u64, //Whats the total amount of liquidity being borrowed for mint_a in USDC
    pub total_borrowed_b: u64, //Whats the total amount of liquidity being borrowed for mint_b in USDC
    pub total_borrowed: u64, //Whats the total borrowed total_borrowed_a + total_borrowed_b in USDC
    pub ltv_ratio: u8, //Loan to value ration 0 - 100
    pub liquidation_threshold: u8, //At what percentage the collateral should be liquidated. 0 - 100
    pub liquidation_penalty: u8, //Penalty applied when liquidating a position. The liquidator receives this bonus, incentivizing them them to perfrom the action
    pub interest_rate: u8, //Annualized interest rate applied to borrowed tokens.
    pub created_at: i64, //Unix timestamp of when the pool was initialized.
    pub lp_supply: u64, //Tracks total LP tokens minted
    pub bump: u8, //Stores the liquidity_pool account bump 
    pub vault_a_bump: u8, //Stores the vault_a account bump
    pub vault_b_bump: u8, //Stores the vault_b account bump
    pub fees_vault_a_bump: u8,
    pub fees_vault_b_bump: u8,
}

impl LiquidityPool {
    pub const LEN: usize = core::mem::size_of::<LiquidityPool>();

    pub fn get_account_info(accounts: &AccountInfo) -> Self {
        return unsafe {*(accounts.borrow_data_unchecked().as_ptr() as *const Self)} ;
    }

    pub fn get_account_info_mut(accounts: &AccountInfo) -> &mut Self {
        return unsafe {&mut *(accounts.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self)} ;
    }
}