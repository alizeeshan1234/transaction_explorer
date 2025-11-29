use anchor_lang::prelude::*;

#[error_code]
pub enum PlatformError {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Overflow in arithmetic operation")]
    MathOverflow,
    #[msg("Unsupported price oracle")]
    UnsupportedOracle,
    #[msg("Invalid oracle account")]
    InvalidOracleAccount,
    #[msg("Invalid oracle state")]
    InvalidOracleState,
    #[msg("Stale oracle price")]
    StaleOraclePrice,
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
    #[msg("Instruction is not allowed in production")]
    InvalidEnvironment,
    #[msg("Invalid platform state")]
    InvalidPlatformState,
    #[msg("Invalid pool state")]
    InvalidPoolState,
    #[msg("Invalid custody state")]
    InvalidCustodyState,
    #[msg("Invalid Market state")]
    InvalidMarketState,
    #[msg("Invalid collateral custody")]
    InvalidCollateralCustody,
    #[msg("Invalid position state")]
    InvalidPositionState,
    #[msg("Invalid Dispensing Custody")]
    InvalidDispensingCustody,
    #[msg("Invalid perpetuals config")]
    InvalidPerpetualsConfig,
    #[msg("Invalid pool config")]
    InvalidPoolConfig,
    #[msg("Invalid custody config")]
    InvalidCustodyConfig,
    #[msg("Invalid basket state")]
    InvalidBasketState,
    #[msg("Insufficient token amount returned")]
    InsufficientAmountReturned,
    #[msg("Price slippage limit exceeded")]
    MaxPriceSlippage,
    #[msg("Position leverage limit exceeded")]
    MaxLeverage,
    #[msg("Position initial leverage limit exceeded")]
    MaxInitLeverage,
    #[msg("Position leverage less than minimum")]
    MinLeverage,
    #[msg("Custody amount limit exceeded")]
    CustodyAmountLimit,
    #[msg("Position amount limit exceeded")]
    PositionAmountLimit,
    #[msg("Token ratio out of range")]
    TokenRatioOutOfRange,
    #[msg("Token is not supported")]
    UnsupportedToken,
    #[msg("Custody is not supported")]
    UnsupportedCustody,
    #[msg("Pool is not supported")]
    UnsupportedPool,
    #[msg("Market is not supported")]
    UnsupportedMarket,
    #[msg("Instruction is not allowed at this time")]
    InstructionNotAllowed,
    #[msg("Token utilization limit exceeded")]
    MaxUtilization,
    #[msg("Close-only mode activated")]
    CloseOnlyMode,
    #[msg("Minimum collateral limit breached")]
    MinCollateral,
    #[msg("Permissionless oracle update must be preceded by Ed25519 signature verification instruction")]
    PermissionlessOracleMissingSignature,
    #[msg("Ed25519 signature verification data does not match expected format")]
    PermissionlessOracleMalformedEd25519Data,
    #[msg("Ed25519 signature was not signed by the oracle authority")]
    PermissionlessOracleSignerMismatch,
    #[msg("Signed message does not match instruction params")]
    PermissionlessOracleMessageMismatch,
    #[msg("Exponent Mismatch betweeen operands")]
    ExponentMismatch,
    #[msg("Invalid Close Ratio")]
    CloseRatio,
    #[msg("Insufficient LP tokens staked")]
    InsufficientStakeAmount,
    #[msg("Invalid Fee Deltas")]
    InvalidFeeDeltas,
    #[msg("Invalid Fee Distrivution Custody")]
    InvalidFeeDistributionCustody,
    #[msg("Invalid Collection")]
    InvalidCollection,
    #[msg("Owner of Token Account does not match")]
    InvalidOwner,
    #[msg("Only nft holders or referred users can trade")]
    InvalidAccess,
    #[msg("Token Stake account doesnot match referral account")]
    TokenStakeAccountMismatch,
    #[msg("Max deposits reached")]
    MaxDepostsReached,
    #[msg("Invalid Stop Loss price")]
    InvalidStopLossPrice,
    #[msg("Invalid Take Profit price")]
    InvalidTakeProfitPrice,
    #[msg("Max exposure limit exceeded for the market")]
    ExposureLimitExceeded,
    #[msg("Stop Loss limit exhausted")]
    MaxStopLossOrders,
    #[msg("Take Profit limit exhausted")]
    MaxTakeProfitOrders,
    #[msg("Open order limit exhausted")]
    MaxOpenOrder,
    #[msg("Invalid Order")]
    InvalidOrder,
    #[msg("Invalid Limit price")]
    InvalidLimitPrice,
    #[msg("Minimum reserve limit breached")]
    MinReserve,
    #[msg("Withdraw Token Request limit exhausted")]
    MaxWithdrawTokenRequest,
    #[msg("Invalid Reward Distribution")]
    InvalidRewardDistribution,
    #[msg("Liquidity Token price is out of bounds")]
    LpPriceOutOfBounds,
    #[msg("Insufficient rebate reserves")]
    InsufficientRebateReserves,
}
