use {
    anchor_lang::prelude::*, 
    anchor_spl::token::*, 
    anchor_spl::associated_token::*,
    crate::collections::{Event, TicketMachine, Ticket}, 
    crate::errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(event_id: String, ticket_machine_id: String, ticket_mint_id: String)]
pub struct CheckIn<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Event account
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
        mut,
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_id.as_bytes(),
        ],
        bump = ticket_machine.bump,
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
    /// Ticket mint
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            ticket_machine.key().as_ref(),
            ticket_mint_id.as_bytes()
        ],
        bump = ticket.mint_bump
    )]
    pub ticket_mint: Box<Account<'info, Mint>>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.metadata_bump,
        seeds::program = metadata_program.key()
    )]
    pub ticket_metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            authority.key().as_ref(),
            token_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.associated_token_bump,
        seeds::program = associated_token_program.key()
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.bump,
        constraint = !ticket.checked_in @ ErrorCode::TicketAlreadyCheckedIn
    )]
    pub ticket: Box<Account<'info, Ticket>>,
}


pub fn handle(ctx: Context<CheckIn>) -> Result<()> {
    (*ctx.accounts.ticket_machine).used += 1;
    (*ctx.accounts.ticket).checked_in = true;

    solana_program::program::invoke(
        &mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::ID,
            (*ctx.accounts.ticket_metadata).key(),
            (*ctx.accounts.ticket_vault).key(),
            (*ctx.accounts.ticket_mint).key(),
            None,
            (*ctx.accounts.authority).key(),
            (*ctx.accounts.authority).key(),
            None,
            1,
        ),
        &[
            ctx.accounts.ticket_metadata.to_account_info().clone(),
            ctx.accounts.ticket_vault.to_account_info().clone(),
            ctx.accounts.ticket_mint.to_account_info().clone(),
            ctx.accounts.authority.to_account_info().clone(),
            ctx.accounts.token_program.to_account_info().clone(),
            ctx.accounts
                .associated_token_program
                .to_account_info()
                .clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone(),
        ],
    )?;

    Ok(())
}