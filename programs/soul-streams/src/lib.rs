use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use utils::*;
pub mod utils;

declare_id!("4SKsW4jTMPcdetYPjFqvZw76LYYcgAJXZEdtxHHXe9vN");

#[program]
pub mod soul_streams {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        emit!(events::Initialized {
            stream_count: ctx.accounts.stream_count.count
        });

        Ok(())
    }

    pub fn create_stream(ctx: Context<CreateStream>, amount: u64, duration: u64) -> Result<()> {
        require!(amount > 0, errors::CustomErrors::ZeroAmount);
        require!(duration > 0, errors::CustomErrors::ZeroDuration);

        // Initialize the payment stream
        let new_stream = &mut ctx.accounts.stream;
        new_stream.payer = ctx.accounts.payer.key();
        new_stream.payee = ctx.accounts.payee.key();
        new_stream.mint = ctx.accounts.mint.key();
        new_stream.amount = amount;
        new_stream.starting_timestamp = ctx.accounts.clock.unix_timestamp as u64;
        new_stream.duration = duration;

        // Transfer tokens from payer to the stream token account
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.stream_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(events::NewStreamCreated {
            payer: new_stream.payer,
            payee: new_stream.payee,
            mint: new_stream.mint,
            amount,
            starting_timestamp: new_stream.starting_timestamp,
            duration
        });

        Ok(())
    }

    pub fn withdraw_from_stream(ctx: Context<WithdrawFromStream>) -> Result<()> {
        Ok(())
    }

    pub fn cancel_stream(ctx: Context<CancelStream>) -> Result<()> {
        Ok(())
    }

    pub fn replenish_stream(ctx: Context<ReplenishStream>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [constants::seeds::STREAM_COUNT],
        bump,
        space = constants::general::DISCRIMINATOR_SPACE + StreamCount::INIT_SPACE
    )]
    pub stream_count: Account<'info, StreamCount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateStream<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: The receiver of the payment stream
    #[account()]
    pub payee: AccountInfo<'info>,
    #[account(
        seeds = [constants::seeds::STREAM_COUNT],
        bump,
    )]
    stream_count: Account<'info, StreamCount>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payer)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(init,
        payer = payer,
        seeds = [constants::seeds::TOKEN_ACCOUNT,
                    payer.key().as_ref(),
                    payee.key().as_ref(),
                    stream_count.count.to_le_bytes().as_ref()
                ],
        bump,
        token::mint = mint,
        token::authority = stream_token_account
    )]
    pub stream_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
        seeds = [constants::seeds::STREAM],
        bump,
        space = constants::general::DISCRIMINATOR_SPACE + Stream::INIT_SPACE
    )]
    pub stream: Account<'info, Stream>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawFromStream<'info> {
    #[account()]
    payee: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelStream<'info> {
    #[account()]
    payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReplenishStream<'info> {
    #[account()]
    payer: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct StreamCount {
    count: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Stream {
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub starting_timestamp: u64,
    pub duration: u64,
    pub streamed_amount_so_far: u64,
}
