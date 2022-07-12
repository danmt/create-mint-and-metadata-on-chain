use anchor_lang::prelude::*;
use anchor_spl::token::*;
use mpl_token_metadata::instruction::*;
use mpl_token_metadata::solana_program::program as solana_program;

declare_id!("EJQnbXhsLS92wsAXg1vPaZt88hfzmuhcqBVLQBn9h23x");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(ctx: Context<CreateEvent>) -> Result<()> {
        (*ctx.accounts.event).bump = *ctx.bumps.get("event").unwrap();
        (*ctx.accounts.event).ticket_mint_bump = *ctx.bumps.get("ticket_mint").unwrap();

        solana_program::invoke(
            &create_metadata_accounts_v2(
                mpl_token_metadata::ID,
                (*ctx.accounts.metadata).key(),
                ctx.accounts.ticket_mint.key(),
                ctx.accounts.event.key(),
                (*ctx.accounts.authority).key(),
                ctx.accounts.event.key(),
                "name".to_string(),
                "symbol".to_string(),
                "uri".to_string(),
                None,
                0,
                true,
                true,
                None,
                None,
            ),
            &[
                ctx.accounts.metadata_program.to_account_info().clone(),
                ctx.accounts.metadata.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
                ctx.accounts.ticket_mint.to_account_info().clone(),
                ctx.accounts.event.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
            ],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateEvent<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::id())]
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
        space = 8 + 1 + 1,
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
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
}

#[account]
pub struct Event {
    pub bump: u8,
    pub ticket_mint_bump: u8,
}
