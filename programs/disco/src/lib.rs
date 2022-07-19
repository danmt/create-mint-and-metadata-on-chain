use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

declare_id!("EJQnbXhsLS92wsAXg1vPaZt88hfzmuhcqBVLQBn9h23x");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(
        ctx: Context<CreateEvent>,
        ticket_title: String,
        ticket_symbol: String,
        ticket_uri: String,
    ) -> Result<()> {
        (*ctx.accounts.event).ticket_title = ticket_title.clone();
        (*ctx.accounts.event).ticket_symbol = ticket_symbol.clone();
        (*ctx.accounts.event).ticket_uri = ticket_uri.clone();
        (*ctx.accounts.event).bump = *ctx.bumps.get("event").unwrap();
        (*ctx.accounts.event).ticket_mint_bump = *ctx.bumps.get("ticket_mint").unwrap();
        (*ctx.accounts.event).ticket_metadata_bump = *ctx.bumps.get("ticket_metadata").unwrap();

        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[*ctx.bumps.get("event").unwrap()],
        ];

        solana_program::program::invoke_signed(
            &mpl_token_metadata::instruction::create_metadata_accounts_v3(
                mpl_token_metadata::ID,
                (*ctx.accounts.ticket_metadata).key(),
                ctx.accounts.ticket_mint.key(),
                ctx.accounts.event.key(),
                (*ctx.accounts.authority).key(),
                ctx.accounts.event.key(),
                ticket_title,
                ticket_symbol,
                ticket_uri,
                None,
                0,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                ctx.accounts.metadata_program.to_account_info().clone(),
                ctx.accounts.ticket_metadata.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
                ctx.accounts.ticket_mint.to_account_info().clone(),
                ctx.accounts.event.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(
    ticket_title: String,
    ticket_symbol: String,
    ticket_uri: String,
)]
pub struct CreateEvent<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 36 + 14 + 204 + 1 + 1 + 1,
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump
    )]
    pub event: Account<'info, Event>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = event,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
        ],
        bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            mpl_token_metadata::ID.as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    pub ticket_metadata: UncheckedAccount<'info>,
}

#[account]
pub struct Event {
    pub ticket_title: String,  // max 32
    pub ticket_symbol: String, // max 10
    pub ticket_uri: String,    // max 200
    pub bump: u8,
    pub ticket_mint_bump: u8,
    pub ticket_metadata_bump: u8,
}
