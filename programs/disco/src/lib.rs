use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("EJQnbXhsLS92wsAXg1vPaZt88hfzmuhcqBVLQBn9h23x");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(
        ctx: Context<CreateEvent>,
        ticket_title: String,
        ticket_symbol: String,
        ticket_uri: String,
        ticket_price: u32,
    ) -> Result<()> {
        (*ctx.accounts.event).ticket_title = ticket_title.clone();
        (*ctx.accounts.event).ticket_symbol = ticket_symbol.clone();
        (*ctx.accounts.event).ticket_uri = ticket_uri.clone();
        (*ctx.accounts.event).ticket_price = ticket_price.clone();
        (*ctx.accounts.event).accepted_mint = ctx.accounts.accepted_mint.key();
        (*ctx.accounts.event).bump = *ctx.bumps.get("event").unwrap();
        (*ctx.accounts.event).ticket_mint_bump = *ctx.bumps.get("ticket_mint").unwrap();
        (*ctx.accounts.event).ticket_metadata_bump = *ctx.bumps.get("ticket_metadata").unwrap();
        (*ctx.accounts.event).event_vault_bump = *ctx.bumps.get("event_vault").unwrap();

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

    pub fn buy_tickets(ctx: Context<BuyTickets>, ticket_quantity: u16) -> Result<()> {
        // call transfer from authority to event vault
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token.to_account_info(),
                    to: ctx.accounts.event_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            ctx.accounts
                .event
                .ticket_price
                .checked_mul(ticket_quantity.into())
                .unwrap()
                .into(),
        )?;

        // call mintTo instruction
        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    to: ctx.accounts.ticket_receiver.to_account_info(),
                    authority: ctx.accounts.event.to_account_info(),
                },
                &[&seeds[..]],
            ),
            ticket_quantity.into(),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(
    ticket_title: String,
    ticket_symbol: String,
    ticket_uri: String,
    ticket_price: u32,
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
        space = 8 + 36 + 14 + 204 + 4 + 32 + 1 + 1 + 1 + 1,
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump
    )]
    pub event: Account<'info, Event>,
    pub accepted_mint: Account<'info, Mint>,
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
            metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub ticket_metadata: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        token::authority = event,
        token::mint = accepted_mint,
        seeds = [
            b"event_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump
    )]
    pub event_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
#[instruction(ticket_quantity: u16)]
pub struct BuyTickets<'info> {
    /// CHECK: this is verified through an address constraint>
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    #[account(
        constraint = accepted_mint.key() == event.accepted_mint
    )]
    pub accepted_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = payer_token.mint == event.accepted_mint
    )]
    pub payer_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"event_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump = event.event_vault_bump
    )]
    pub event_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
        ],
        bump = event.ticket_mint_bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = authority,
        token::authority = authority,
        token::mint = ticket_mint,
        seeds = [
            b"ticket_receiver".as_ref(),
            authority.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump
    )]
    pub ticket_receiver: Box<Account<'info, TokenAccount>>,
}

#[account]
pub struct Event {
    pub ticket_title: String,  // max 32
    pub ticket_symbol: String, // max 10
    pub ticket_uri: String,    // max 200
    pub ticket_price: u32,
    pub accepted_mint: Pubkey,
    pub bump: u8,
    pub event_vault_bump: u8,
    pub ticket_mint_bump: u8,
    pub ticket_metadata_bump: u8,
}
