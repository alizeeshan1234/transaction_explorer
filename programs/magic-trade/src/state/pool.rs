use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    custody::Custody,
    error::PlatformError,
    market::{Market, OraclePrice, Position, Side},
    math, MAX_CUSTODIES, MAX_MARKETS, RATE_POWER,
};

#[account]
#[derive(Default, InitSpace, Debug)]
pub struct Pool {
    pub id: u8,
    pub pool_bump: u8,
    pub lp_mint_bump: u8,
    pub custody_count: u8,
    pub staleness_threshold: u8,
    pub padding: [u8; 3],
    pub max_aum_usd: u64,
    pub buffer: u64,
    pub raw_aum_usd: u64,
    pub equity_usd: u64,
    pub last_updated_at: i64,
    pub collateral_oracle: Pubkey,
    #[max_len(MAX_CUSTODIES as usize)]
    pub custodies: Vec<Pubkey>,
    #[max_len(MAX_MARKETS as usize)]
    pub markets: Vec<Pubkey>,
}

impl Pool {
    pub fn get_fee_value(&self, fee: u64, amount: u64) -> Result<u64> {
        if fee == 0 || amount == 0 {
            return Ok(0);
        }
        math::checked_as_u64(math::checked_ceil_div(
            math::checked_mul(amount as u128, fee as u128)?,
            RATE_POWER,
        )?)
    }

    pub fn update_aum<'info>(
        &mut self,
        accounts: &'info [AccountInfo<'info>],
        curtime: i64,
    ) -> Result<(u64, u64)> {
        let (mut aum, mut profit, mut loss) = (0u64, 0u64, 0u64);
        let oracle_offset = self.custodies.len();
        for (cid, &custody_key) in self.custodies.iter().enumerate() {
            let custody_account = &accounts[cid];
            require_eq!(
                custody_account.key(),
                custody_key,
                PlatformError::InvalidCustodyState
            );
            let custody: Account<Custody> = Account::try_from(custody_account)?;
            // let price_data = PriceUpdateV2::try_deserialize_unchecked(
            //     &mut (accounts[oracle_offset + cid].data.borrow()).as_ref(),
            // ).map_err(Into::<Error>::into)?.price_message;
            // let price = OraclePrice::new(price_data.price.try_into().unwrap(), price_data.exponent);
            let oracle_account = &accounts[oracle_offset + cid];
            require_eq!(
                oracle_account.key(),
                custody.oracle,
                PlatformError::InvalidOracleAccount
            );
            let price =
                OraclePrice::from_pyth(oracle_account, curtime, custody.max_price_age as i64)?;
            aum = math::checked_add(
                aum,
                price.get_asset_amount_usd(custody.assets.owned, custody.decimals)?,
            )?;
            for &market in custody.supported_markets.iter() {
                let market_account = accounts
                    .iter()
                    .find(|acc| acc.key() == market)
                    .ok_or(PlatformError::InvalidMarketState)?;
                let market: Account<Market> = Account::try_from(market_account)?;
                let (cp_profit, cp_loss) = market.collective_position.get_pnl_usd(
                    market.side,
                    &price,
                    curtime,
                    custody.margin_params.virtual_delay,
                )?;
                profit = math::checked_add(profit, cp_loss)?;
                loss = math::checked_add(loss, cp_profit)?;
            }
        }
        self.raw_aum_usd = aum;
        self.equity_usd = aum.saturating_add(profit).saturating_sub(loss);

        return Ok((aum, self.equity_usd));
    }
}
