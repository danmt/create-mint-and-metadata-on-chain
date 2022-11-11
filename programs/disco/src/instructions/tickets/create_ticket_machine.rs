use {
    anchor_lang::prelude::*,
    anchor_spl::token::Token,
    crate::collections::{Event, TicketMachine}
};

#[derive(Accounts)]
#[instruction(
    ticket_name: String,
    ticket_symbol: String,
    ticket_uri: String,
    ticket_price: u64,
    ticket_quantity: u64,
    ticket_uses: u64,
    event_id: String,
    ticket_id: String
)]
pub struct CreateTicketMachine<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Event
    #[account(
        seeds = [
            b"event".as_ref(),
            event_id.as_bytes(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    /// Ticket Machine
    #[account(
        init,
        payer = authority,
        space = TicketMachine::SIZE,
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_id.as_bytes(),
        ],
        bump
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
}

pub fn handle(
    ctx: Context<CreateTicketMachine>,
    ticket_name: String,
    ticket_symbol: String,
    ticket_uri: String,
    ticket_price: u64,
    ticket_quantity: u64,
    ticket_uses: u64,
) -> Result<()> {
    (*ctx.accounts.ticket_machine).name = ticket_name;
    (*ctx.accounts.ticket_machine).symbol = ticket_symbol;
    (*ctx.accounts.ticket_machine).uri = ticket_uri;
    (*ctx.accounts.ticket_machine).quantity = ticket_quantity;
    (*ctx.accounts.ticket_machine).price = ticket_price;
    (*ctx.accounts.ticket_machine).uses = ticket_uses;
    (*ctx.accounts.ticket_machine).sold = 0;
    (*ctx.accounts.ticket_machine).used = 0;
    (*ctx.accounts.ticket_machine).bump = *ctx.bumps.get("ticket_machine").unwrap();

    Ok(())
}