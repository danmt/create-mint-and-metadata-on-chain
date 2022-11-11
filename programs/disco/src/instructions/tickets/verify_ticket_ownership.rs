use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{ Mint, Token, TokenAccount},
    },
    crate::collections::{Event,TicketMachine,Ticket},
    crate::errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(event_id: String, ticket_machine_id:String, ticket_mint_id: String)]
pub struct VerifyTicketOwnership<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub authority: Signer<'info>,
    // Event
    #[account(
        seeds = [
            b"event".as_ref(),
            event_id.as_bytes()
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    /// Ticket Machine
    #[account(
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_id.as_bytes(),
        ],
        bump = ticket_machine.bump,
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
    /// Ticket Mint
    #[account(
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            ticket_machine.key().as_ref(),
            ticket_mint_id.as_bytes()
        ],
        bump = ticket.mint_bump
    )]
    pub ticket_mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.bump,
        constraint = ticket.authority == authority.key() @ ErrorCode::InvalidAuthorityForTicket
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    #[account(
        seeds = [
            authority.key().as_ref(),
            token_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.associated_token_bump,
        seeds::program = associated_token_program.key(),
        constraint = ticket_vault.amount > 0 @ ErrorCode::InvalidAuthorityForTicket,
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
}

pub fn handle(_ctx: Context<VerifyTicketOwnership>) -> Result<()> {
    Ok(())
}