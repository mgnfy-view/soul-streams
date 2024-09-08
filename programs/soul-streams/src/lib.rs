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

    pub fn create_stream(
        ctx: Context<CreateStream>,
        payee: Pubkey,
        amount: u64,
        starting_timestamp: u64,
        duration: u64,
    ) -> Result<()> {
        require!(amount > 0, errors::CustomErrors::ZeroAmount);
        require!(
            starting_timestamp >= ctx.accounts.clock.unix_timestamp as u64,
            errors::CustomErrors::InvalidTimestamp
        );
        require!(duration > 0, errors::CustomErrors::ZeroDuration);

        // Initialize the payment stream
        let new_stream = &mut ctx.accounts.stream;
        new_stream.payer = ctx.accounts.payer.key();
        new_stream.payee = payee.key();
        new_stream.mint = ctx.accounts.mint.key();
        new_stream.amount = amount;
        new_stream.starting_timestamp = starting_timestamp;
        new_stream.duration = duration;
        new_stream.count = ctx.accounts.stream_count.count;

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
            duration
        });

        Ok(())
    }

    pub fn withdraw_from_stream(
        ctx: Context<WithdrawFromStream>,
        payer: Pubkey,
        count: u64,
    ) -> Result<()> {
        let stream = &mut ctx.accounts.stream;
        let time_passed_so_far =
            ctx.accounts.clock.unix_timestamp as u64 - stream.starting_timestamp;
        let mut amount_to_emit = utils::get_amount_to_emit(
            &(stream.amount as u128),
            &(stream.duration as u128),
            &(time_passed_so_far as u128),
        ) - stream.streamed_amount_so_far;

        if amount_to_emit > stream.amount {
            amount_to_emit = stream.amount;
        }

        require!(amount_to_emit > 0, errors::CustomErrors::ZeroAmountToEmit);

        stream.streamed_amount_so_far += amount_to_emit;

        let payer_key = payer.key().clone();
        let payee_key = ctx.accounts.payee.key().clone();
        let mint_key = ctx.accounts.mint.key().clone();
        let stream_count_bytes = count.to_be_bytes().clone();
        let stream_token_account_seeds = &[
            constants::seeds::TOKEN_ACCOUNT,
            payer_key.as_ref(),
            payee_key.as_ref(),
            mint_key.as_ref(),
            stream_count_bytes.as_ref(),
            &[ctx.bumps.stream_token_account],
        ];
        let stream_token_account_signer = [&stream_token_account_seeds[..]];
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
            amount: amount_to_emit
        });

        Ok(())
    }

    pub fn cancel_stream(ctx: Context<CancelStream>, payee: Pubkey, count: u64) -> Result<()> {
        let stream = &mut ctx.accounts.stream;
        let time_passed_so_far =
            ctx.accounts.clock.unix_timestamp as u64 - stream.starting_timestamp;
        let mut amount_to_emit = utils::get_amount_to_emit(
            &(stream.amount as u128),
            &(stream.duration as u128),
            &(time_passed_so_far as u128),
        ) - stream.streamed_amount_so_far;

        if amount_to_emit > stream.amount {
            amount_to_emit = stream.amount;
        }

        stream.streamed_amount_so_far += amount_to_emit;

        let payer_key = ctx.accounts.payer.key().clone();
        let payee_key = payee.key().clone();
        let mint_key = ctx.accounts.mint.key().clone();
        let stream_count_bytes = count.to_be_bytes().clone();
        let stream_token_account_seeds = &[
            constants::seeds::TOKEN_ACCOUNT,
            payer_key.as_ref(),
            payee_key.as_ref(),
            mint_key.as_ref(),
            stream_count_bytes.as_ref(),
            &[ctx.bumps.stream_token_account],
        ];
        let stream_token_account_signer = [&stream_token_account_seeds[..]];
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

        if amount_to_emit > 0 {
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
            stream: stream.key()
        });

        Ok(())
    }

    pub fn replenish_stream(
        ctx: Context<ReplenishStream>,
        payee: Pubkey,
        count: u64,
    ) -> Result<()> {
        // let stream = &mut ctx.accounts.stream;

        // require!(
        //     stream.starting_timestamp + stream.duration < ctx.accounts.clock.unix_timestamp as u64,
        //     errors::CustomErrors::OngoingStream
        // );

        // let time_passed_so_far =
        //     ctx.accounts.clock.unix_timestamp as u64 - stream.starting_timestamp;
        // let amount_to_emit = utils::get_amount_to_emit(
        //     &(stream.amount as u128),
        //     &(stream.duration as u128),
        //     &(time_passed_so_far as u128)
        // ) - stream.streamed_amount_so_far;

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
    #[account(mut, associated_token::mint = mint, associated_token::authority = payee)]
    pub payee_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
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
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
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
pub struct ReplenishStream<'info> {
    #[account()]
    payer: Signer<'info>,
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
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
        seeds = [
            constants::seeds::STREAM,
            payer.key().as_ref(),
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
