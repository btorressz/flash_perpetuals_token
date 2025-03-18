use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("4YevqLFE6GSj7ERoK5u9LjTyLtJBFVJiX9WUFbnudfp5"); // Replace with your program ID

#[program]
pub mod flash_perpetuals_token {
    use super::*;

    /// Initializes the global state.
    /// Only the admin can call this function.
    pub fn initialize(
        ctx: Context<Initialize>,
        fee_rate: u64,
        maintenance_margin: u64,   // e.g., 150 for 150%
        min_stake_duration: i64,   // in seconds
        execution_fee: u64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.admin = ctx.accounts.admin.key();
        global_state.fee_rate = fee_rate;
        global_state.maintenance_margin = maintenance_margin;
        global_state.min_stake_duration = min_stake_duration;
        global_state.execution_fee = execution_fee;
        global_state.total_staked = 0;
        global_state.total_liquidity = 0;
        global_state.orderbook_merkle_root = [0u8; 32];
        global_state.authorized_liquidators = Vec::new();
        global_state.authorized_orderbook_updaters = Vec::new();
        global_state.reputation_based_leverage = Vec::new();
        Ok(())
    }

    /// Adds an authorized liquidator (e.g., HFT firms, LPs).
    pub fn add_authorized_liquidator(
        ctx: Context<AdminOnly>,
        new_liquidator: Pubkey,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authorized_liquidators.push(new_liquidator);
        Ok(())
    }

    /// Adds an authorized orderbook updater.
    pub fn add_authorized_orderbook_updater(
        ctx: Context<AdminOnly>,
        new_updater: Pubkey,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authorized_orderbook_updaters.push(new_updater);
        Ok(())
    }

    /// Stake tokens to gain access to leveraged positions.
    /// Records the stake timestamp to enforce a time lock.
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        // Transfer tokens from the user to the vault.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update the trader state with the staked amount and current timestamp.
        let trader_state = &mut ctx.accounts.trader_state;
        trader_state.owner = ctx.accounts.user.key();
        trader_state.staked_amount = trader_state
            .staked_amount
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        trader_state.stake_timestamp = clock.unix_timestamp;
        Ok(())
    }

    /// Opens a leveraged perpetual position.
    /// Deducts an execution fee and enforces a time lock to resist flash loan exploits.
    pub fn open_position(
        ctx: Context<OpenPosition>,
        leverage: u64,
        amount: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let trader_state = &mut ctx.accounts.trader_state;
        let global_state = &mut ctx.accounts.global_state;

        require!(
            clock.unix_timestamp >= trader_state.stake_timestamp + global_state.min_stake_duration,
            ErrorCode::StakeTimeLock
        );

        require!(
            trader_state.staked_amount >= global_state.execution_fee,
            ErrorCode::InsufficientFundsForFee
        );
        // Deduct the execution fee.
        trader_state.staked_amount = trader_state
            .staked_amount
            .checked_sub(global_state.execution_fee)
            .ok_or(ErrorCode::MathOverflow)?;
        global_state.total_liquidity = global_state
            .total_liquidity
            .checked_add(global_state.execution_fee)
            .ok_or(ErrorCode::MathOverflow)?;

        // Calculate and record the leveraged position.
        let position_value = amount
            .checked_mul(leverage)
            .ok_or(ErrorCode::MathOverflow)?;
        trader_state.open_position = trader_state
            .open_position
            .checked_add(position_value)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }

    /// Applies the funding rate by deducting a fee from the trader's open position.
    pub fn apply_funding_rate(ctx: Context<ApplyFundingRate>) -> Result<()> {
        let trader_state = &mut ctx.accounts.trader_state;
        let global_state = &mut ctx.accounts.global_state;
        let funding_fee = trader_state
            .open_position
            .checked_mul(global_state.fee_rate)
            .ok_or(ErrorCode::MathOverflow)?;
        trader_state.open_position = trader_state
            .open_position
            .checked_sub(funding_fee)
            .ok_or(ErrorCode::MathOverflow)?;
        global_state.total_liquidity = global_state
            .total_liquidity
            .checked_add(funding_fee)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }

    /// Dynamically adjusts the funding rate.
    /// Can be called by an admin to prevent manipulation.
    pub fn adjust_funding_rate(ctx: Context<AdminOnly>, new_funding_rate: u64) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.fee_rate = new_funding_rate;
        Ok(())
    }

    /// Partial liquidation with penalty and bounty.
    /// Liquidators receive a reward while a portion of the penalty funds the liquidity pool.
    pub fn liquidate_with_penalty(
        ctx: Context<AuthorizedLiquidator>,
        liquidation_amount: u64,
        penalty_rate: u64, // e.g., 5 for a 5% penalty
    ) -> Result<()> {
        let trader_state = &mut ctx.accounts.trader_state;
        let global_state = &mut ctx.accounts.global_state;

        require!(
            global_state
                .authorized_liquidators
                .contains(&ctx.accounts.liquidator.key()),
            ErrorCode::Unauthorized
        );

        // Calculate the required margin: open_position * maintenance_margin / 100.
        let required_margin = trader_state
            .open_position
            .checked_mul(global_state.maintenance_margin)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::MathOverflow)?;
        require!(
            trader_state.staked_amount < required_margin,
            ErrorCode::MarginSufficient
        );

        require!(
            trader_state.open_position >= liquidation_amount,
            ErrorCode::InvalidLiquidationAmount
        );

        // Calculate penalty and split between liquidator and liquidity pool.
        let penalty = liquidation_amount
            .checked_mul(penalty_rate)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::MathOverflow)?;
        let liquidator_reward = penalty
            .checked_div(2)
            .ok_or(ErrorCode::MathOverflow)?;
        let liquidity_pool_bonus = penalty
            .checked_sub(liquidator_reward)
            .ok_or(ErrorCode::MathOverflow)?;

        trader_state.open_position = trader_state
            .open_position
            .checked_sub(liquidation_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        global_state.total_liquidity = global_state
            .total_liquidity
            .checked_add(liquidity_pool_bonus)
            .ok_or(ErrorCode::MathOverflow)?;

        // In production, implement token transfer to reward the liquidator.
        msg!(
            "Liquidator {} receives {} tokens as liquidation bounty",
            ctx.accounts.liquidator.key(),
            liquidator_reward
        );
        Ok(())
    }

    /// Batch execution for HFT: processes multiple trades in one transaction.
    pub fn batch_execute_trades(
        ctx: Context<BatchTradeExecution>,
        trades: Vec<(u64, u64)>, // Vec<(leverage, amount)>
    ) -> Result<()> {
        let trader_state = &mut ctx.accounts.trader_state;
        let mut total_position: u64 = 0;
        for (leverage, amount) in trades.iter() {
            total_position = total_position
                .checked_add(
                    amount
                        .checked_mul(*leverage)
                        .ok_or(ErrorCode::MathOverflow)?,
                )
                .ok_or(ErrorCode::MathOverflow)?;
        }
        trader_state.open_position = trader_state
            .open_position
            .checked_add(total_position)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }

    /// Auto-hedging mechanism for real-time risk management.
    pub fn auto_hedge(ctx: Context<AutoHedge>, hedge_amount: u64) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(
            global_state.total_liquidity >= hedge_amount,
            ErrorCode::InsufficientLiquidity
        );
        global_state.total_liquidity = global_state
            .total_liquidity
            .checked_sub(hedge_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        msg!("Auto hedge executed for {} tokens", hedge_amount);
        Ok(())
    }

    /// Adjusts a trader's leverage limit based on their reputation.
    pub fn adjust_leverage_based_on_reputation(
        ctx: Context<AdjustLeverage>,
        trader: Pubkey,
        new_leverage: u64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        let mut found = false;
        for rep in global_state.reputation_based_leverage.iter_mut() {
            if rep.trader == trader {
                rep.leverage = new_leverage;
                found = true;
                break;
            }
        }
        if !found {
            global_state.reputation_based_leverage.push(ReputationLeverage {
                trader,
                leverage: new_leverage,
            });
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Global state account for program-wide configuration.
    #[account(init, payer = admin, space = 8 + GlobalState::LEN)]
    pub global_state: Account<'info, GlobalState>,
    /// Admin account initializing the program.
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Admin-only context for functions like adding authorized entities and adjusting the funding rate.
#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// The Stake context creates a new TraderState account.
/// (Using `init` instead of `init_if_needed` so that the Cargo feature is not required.)
#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(init, payer = user, space = 8 + TraderState::LEN)]
    pub trader_state: Account<'info, TraderState>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// The user's token account holding FPT tokens.
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    /// The vault account where staked tokens are held.
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    /// Trader state must be owned by the signer.
    #[account(mut, has_one = owner)]
    pub trader_state: Account<'info, TraderState>,
    /// Global state is updated for fee collection.
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct AuthorizedLiquidator<'info> {
    #[account(mut)]
    pub trader_state: Account<'info, TraderState>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    pub liquidator: Signer<'info>,
}

#[derive(Accounts)]
pub struct ApplyFundingRate<'info> {
    #[account(mut)]
    pub trader_state: Account<'info, TraderState>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
pub struct BatchTradeExecution<'info> {
    #[account(mut, has_one = owner)]
    pub trader_state: Account<'info, TraderState>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct AutoHedge<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdjustLeverage<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub fee_rate: u64,
    pub maintenance_margin: u64,
    pub min_stake_duration: i64,
    pub execution_fee: u64,
    pub total_staked: u64,
    pub total_liquidity: u64,
    pub orderbook_merkle_root: [u8; 32],
    pub authorized_liquidators: Vec<Pubkey>,
    pub authorized_orderbook_updaters: Vec<Pubkey>,
    pub reputation_based_leverage: Vec<ReputationLeverage>,
}

impl GlobalState {
    // Calculate space for GlobalState.
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 8 + 8 + 32
        + (4 + 10 * 32)
        + (4 + 10 * 32)
        + (4 + 10 * (32 + 8));
}

#[account]
pub struct TraderState {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub open_position: u64,
    pub stake_timestamp: i64,
}

impl TraderState {
    pub const LEN: usize = 32 + 8 + 8 + 8;
}

#[account]
pub struct LpState {
    pub owner: Pubkey,
    pub lp_staked: u64,
    pub trading_volume: u64,
    pub lp_rewards: u64,
}

impl LpState {
    pub const LEN: usize = 32 + 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReputationLeverage {
    pub trader: Pubkey,
    pub leverage: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Insufficient staked funds for execution fee")]
    InsufficientFundsForFee,
    #[msg("Stake duration has not been met")]
    StakeTimeLock,
    #[msg("Margin is sufficient; liquidation not allowed")]
    MarginSufficient,
    #[msg("Invalid liquidation amount")]
    InvalidLiquidationAmount,
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Insufficient liquidity for hedging")]
    InsufficientLiquidity,
}
