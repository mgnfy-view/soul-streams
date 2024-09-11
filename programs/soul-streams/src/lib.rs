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
        // This count serves as salt for creating new streams
        ctx.accounts.stream_count.count = 1;

        emit!(events::Initialized {
            stream_count: ctx.accounts.stream_count.count
        });

        Ok(())
    }

    pub fn create_stream(
        ctx: Context<CreateStream>,
        payee: Pubkey,
        amount: u64,
        starting_timestamp: u64,
        duration: u64,
    ) -> Result<()> {
        // Some sanity checks
        require!(amount > 0, errors::CustomErrors::ZeroAmount);
        require!(
            starting_timestamp >= ctx.accounts.clock.unix_timestamp as u64,
            errors::CustomErrors::InvalidTimestamp
        );
        require!(duration > 0, errors::CustomErrors::ZeroDuration);

        // Initialize the payment stream
        let new_stream = &mut ctx.accounts.stream;
        new_stream.payer = ctx.accounts.payer.key();
        new_stream.payee = payee;
        new_stream.mint = ctx.accounts.mint.key();
        new_stream.amount = amount;
        new_stream.starting_timestamp = starting_timestamp;
        new_stream.duration = duration;
        new_stream.count = ctx.accounts.stream_count.count;

        // Increment the stream count
        ctx.accounts.stream_count.count += 1;

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
            stream: new_stream.key(),
            payer: new_stream.payer,
            payee: new_stream.payee,
            mint: new_stream.mint,
            amount,
            starting_timestamp: new_stream.starting_timestamp,
            duration,
            count: new_stream.count
        });

        Ok(())
    }

    pub fn withdraw_from_stream(
        ctx: Context<WithdrawFromStream>,
        payer: Pubkey,
        count: u64,
    ) -> Result<()> {
        // Calculate the amount that the payee is eligible to withdraw
        let stream = &mut ctx.accounts.stream;
        let time_passed_so_far =
            ctx.accounts.clock.unix_timestamp as u64 - stream.starting_timestamp;
        let mut amount_to_emit = utils::get_amount_to_emit(
            &(stream.amount as u128),
            &(stream.duration as u128),
            &(time_passed_so_far as u128),
        ) - stream.streamed_amount_so_far;

        require!(amount_to_emit > 0, errors::CustomErrors::ZeroAmountToEmit);

        // If the streaming duration has ended, the amount to emit may be larger than
        // the actual amount available in the stream if the payee hasn't withdrawn much
        // Adjust that
        if amount_to_emit > stream.amount {
            amount_to_emit = stream.amount;
        }

        stream.streamed_amount_so_far += amount_to_emit;

        // Use the stream token account's seeds to transfer funds out
        let payer_key = payer.clone();
        let payee_key = ctx.accounts.payee.key().clone();
        let mint_key = ctx.accounts.mint.key().clone();
        let stream_count_bytes = count.to_le_bytes().clone();
        let stream_token_account_seeds = &[
            constants::seeds::TOKEN_ACCOUNT,
            payer_key.as_ref(),
            payee_key.as_ref(),
            mint_key.as_ref(),
            stream_count_bytes.as_ref(),
            &[ctx.bumps.stream_token_account],
        ];
        let stream_token_account_signer = [&stream_token_account_seeds[..]];

        // Transfer the tokens to the payee
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.stream_token_account.to_account_info(),
                    to: ctx.accounts.payee_token_account.to_account_info(),
                    authority: ctx.accounts.stream_token_account.to_account_info(),
                },
                &stream_token_account_signer,
            ),
            amount_to_emit,
        )?;

        emit!(AmountWithdrawnFromStream {
            stream: stream.key(),
            payer: stream.payer,
            payee: stream.payee,
            mint: stream.mint,
            amount_withdrawn: amount_to_emit,
            count: stream.count
        });

        Ok(())
    }

    pub fn cancel_stream(ctx: Context<CancelStream>, payee: Pubkey, count: u64) -> Result<()> {
        // Calculate the amount that's entitled to the payee
        let stream = &mut ctx.accounts.stream;
        let time_passed_so_far =
            ctx.accounts.clock.unix_timestamp as u64 - stream.starting_timestamp;
        let mut amount_to_emit = utils::get_amount_to_emit(
            &(stream.amount as u128),
            &(stream.duration as u128),
            &(time_passed_so_far as u128),
        ) - stream.streamed_amount_so_far;

        // Just as before, it's possible that the stream is being cancelled after the duration
        // has ended. In that case, if the payee hasn't withdrawn much, they're able to withdraw
        // the entire stream balance
        if amount_to_emit > stream.amount {
            amount_to_emit = stream.amount;
        }

        stream.streamed_amount_so_far += amount_to_emit;

        // Get the stream token account's seeds
        let payer_key = ctx.accounts.payer.key().clone();
        let payee_key = payee.clone();
        let mint_key = ctx.accounts.mint.key().clone();
        let stream_count_bytes = count.to_le_bytes().clone();
        let stream_token_account_seeds = &[
            constants::seeds::TOKEN_ACCOUNT,
            payer_key.as_ref(),
            payee_key.as_ref(),
            mint_key.as_ref(),
            stream_count_bytes.as_ref(),
            &[ctx.bumps.stream_token_account],
        ];
        let stream_token_account_signer = [&stream_token_account_seeds[..]];

        // Stream the amount that the payee is entitled to
        if amount_to_emit > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_token_account.to_account_info(),
                        to: ctx.accounts.payee_token_account.to_account_info(),
                        authority: ctx.accounts.stream_token_account.to_account_info(),
                    },
                    &stream_token_account_signer,
                ),
                amount_to_emit,
            )?;
        }

        let remaining_amount = stream.amount - stream.streamed_amount_so_far;

        // Transfer any remaining balance back to the payer
        if remaining_amount > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_token_account.to_account_info(),
                        to: ctx.accounts.payer_token_account.to_account_info(),
                        authority: ctx.accounts.stream_token_account.to_account_info(),
                    },
                    &stream_token_account_signer,
                ),
                remaining_amount,
            )?;
        }

        emit!(events::StreamCanceled {
            stream: stream.key(),
            payer: stream.payer,
            payee: stream.payee,
            mint: stream.mint,
            count: stream.count
        });

        Ok(())
    }

    pub fn replenish_stream(
        ctx: Context<ReplenishStream>,
        payee: Pubkey,
        count: u64,
        new_amount: u64,
        new_duration: u64,
        new_starting_timestamp: u64,
    ) -> Result<()> {
        let stream = &mut ctx.accounts.stream;

        // Ensure that the stream has ended
        require!(
            stream.starting_timestamp + stream.duration < ctx.accounts.clock.unix_timestamp as u64,
            errors::CustomErrors::OngoingStream
        );

        // Get the stream token account's seeds
        let payer_key = ctx.accounts.payer.key().clone();
        let payee_key = payee.clone();
        let mint_key = ctx.accounts.mint.key().clone();
        let stream_count_bytes = count.to_le_bytes().clone();
        let stream_token_account_seeds = &[
            constants::seeds::TOKEN_ACCOUNT,
            payer_key.as_ref(),
            payee_key.as_ref(),
            mint_key.as_ref(),
            stream_count_bytes.as_ref(),
            &[ctx.bumps.stream_token_account],
        ];
        let stream_token_account_signer = [&stream_token_account_seeds[..]];

        // If the payee hasn't withdrawn funds yet, transfer the remaining funds to them
        if stream.streamed_amount_so_far < stream.amount {
            let remaining_amount = stream.amount - stream.streamed_amount_so_far;

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_token_account.to_account_info(),
                        to: ctx.accounts.payer_token_account.to_account_info(),
                        authority: ctx.accounts.stream_token_account.to_account_info(),
                    },
                    &stream_token_account_signer,
                ),
                remaining_amount,
            )?;
        }

        // Sanity checks for new params
        require!(new_duration > 0, errors::CustomErrors::ZeroDuration);
        require!(new_amount > 0, errors::CustomErrors::ZeroAmount);
        require!(
            new_starting_timestamp >= ctx.accounts.clock.unix_timestamp as u64,
            errors::CustomErrors::InvalidTimestamp
        );

        // Update the stream
        stream.amount = new_amount;
        stream.streamed_amount_so_far = 0;
        stream.duration = new_duration;
        stream.starting_timestamp = new_starting_timestamp;

        // Fund the updated stream
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.stream_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            new_amount,
        )?;

        emit!(events::StreamReplenished {
            stream: stream.key(),
            payer: ctx.accounts.payer.key(),
            payee,
            mint: ctx.accounts.mint.key(),
            amount: new_amount,
            starting_timestamp: new_starting_timestamp,
            duration: new_duration
        });

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
#[instruction(
    payee: Pubkey,
    amount: u64,
    starting_timestamp: u64,
    duration: u64,
)]
pub struct CreateStream<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [constants::seeds::STREAM_COUNT],
        bump,
    )]
    stream_count: Account<'info, StreamCount>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payer)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
        seeds = [
            constants::seeds::TOKEN_ACCOUNT,
            payer.key().as_ref(),
            payee.key().as_ref(),
            mint.key().as_ref(),
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
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
            payee.key().as_ref(),
            mint.key().as_ref(),
            stream_count.count.to_le_bytes().as_ref()
        ],
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
#[instruction(payer: Pubkey, count: u64)]
pub struct WithdrawFromStream<'info> {
    #[account()]
    pub payee: Signer<'info>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            constants::seeds::TOKEN_ACCOUNT,
            payer.as_ref(),
            payee.key().as_ref(),
            mint.key().as_ref(),
            stream.count.to_le_bytes().as_ref()
        ],
        bump,
        token::mint = mint,
        token::authority = stream_token_account
    )]
    pub stream_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payee)]
    pub payee_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            constants::seeds::STREAM,
            payer.as_ref(),
            payee.key().as_ref(),
            mint.key().as_ref(),
            count.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub stream: Box<Account<'info, Stream>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(payee: Pubkey, count: u64)]
pub struct CancelStream<'info> {
    #[account()]
    payer: Signer<'info>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            constants::seeds::TOKEN_ACCOUNT,
            payer.key().as_ref(),
            payee.key().as_ref(),
            mint.key().as_ref(),
            stream.count.to_le_bytes().as_ref()
        ],
        bump,
        token::mint = mint,
        token::authority = stream_token_account
    )]
    pub stream_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payer)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payee)]
    pub payee_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
            payee.as_ref(),
            mint.key().as_ref(),
            count.to_le_bytes().as_ref()
        ],
        bump,
        close = payer
    )]
    pub stream: Box<Account<'info, Stream>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(
    payee: Pubkey,
    count: u64,
    new_amount: u64,
    new_duration: u64,
    new_starting_timestamp: u64
)]
pub struct ReplenishStream<'info> {
    #[account()]
    payer: Signer<'info>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            constants::seeds::TOKEN_ACCOUNT,
            payer.key().as_ref(),
            payee.as_ref(),
            mint.key().as_ref(),
            stream.count.to_le_bytes().as_ref()
        ],
        bump,
        token::mint = mint,
        token::authority = stream_token_account
    )]
    pub stream_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payer)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = payee)]
    pub payee_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
            payee.as_ref(),
            mint.key().as_ref(),
            count.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub stream: Box<Account<'info, Stream>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
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
    pub count: u64,
}
