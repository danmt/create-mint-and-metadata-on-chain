use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint,Token, TokenAccount},
    },
    crate::collections::{Ticket},
    crate::errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(new_authority_ticket_vault_bump: u8)]
pub struct SetTicketAuthority<'info> {
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub authority: Signer<'info>,
    /// CHECK: new authority can be anything.
    pub new_authority: UncheckedAccount<'info>,
    pub ticket_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.bump,
        constraint = ticket.authority == authority.key() @ ErrorCode::OnlyTicketAuthorityCanChangeAuthority,
        constraint = !ticket.checked_in @ ErrorCode::CheckedInTicketsCantChangeAuthority,
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    #[account(
        seeds = [
            new_authority.key().as_ref(),
            token_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = new_authority_ticket_vault_bump,
        seeds::program = associated_token_program.key()
    )]
    pub new_authority_ticket_vault: Box<Account<'info, TokenAccount>>,
}

pub fn handle(
    ctx: Context<SetTicketAuthority>,
    new_authority_ticket_vault_bump: u8,
) -> Result<()> {
    (*ctx.accounts.ticket).authority = ctx.accounts.new_authority.key();
    (*ctx.accounts.ticket).associated_token_bump = new_authority_ticket_vault_bump;

    Ok(())
}