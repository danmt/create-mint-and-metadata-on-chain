use anchor_lang::prelude::*;

mod collections;
mod errors;
mod instructions;
use instructions::*;

declare_id!("4tbPa4djkjFpGbK6CR1cS7ZN8bkRet7gydz8SHwM4ZTb");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(
        ctx: Context<CreateEvent>,
        event_name: String,
        event_symbol: String,
        event_uri: String,
        event_id: String
    ) -> Result<()> {
       instructions::events::create_event::handle(ctx, event_name, event_symbol, event_uri, event_id)
    }

    pub fn create_collaborator(ctx: Context<CreateCollaborator>) -> Result<()> {
        instructions::collaborators::create_collaborator::handle(ctx)
    }

    pub fn delete_collaborator(ctx: Context<DeleteCollaborator>) -> Result<()> {
        instructions::collaborators::delete_collaborator::handle(ctx)
    }

    pub fn create_ticket_machine(
        ctx: Context<CreateTicketMachine>,
        ticket_name: String,
        ticket_symbol: String,
        ticket_uri: String,
        ticket_price: u64,
        ticket_quantity: u64,
        ticket_uses: u64,
    ) -> Result<()> {
        instructions::tickets::create_ticket_machine::handle(ctx, ticket_name, ticket_symbol, ticket_uri, ticket_price, ticket_quantity, ticket_uses)
    }

    pub fn mint_ticket(ctx: Context<MintTicket>, ticket_vault_bump: u8) -> Result<()> {
        instructions::tickets::mint_ticket::handle(ctx, ticket_vault_bump)
    }

    pub fn check_in(ctx: Context<CheckIn>) -> Result<()> {
        instructions::events::check_in::handle(ctx)
    }

    pub fn verify_ticket_ownership(ctx: Context<VerifyTicketOwnership>) -> Result<()> {
        instructions::tickets::verify_ticket_ownership::handle(ctx)
    }

    pub fn set_ticket_authority(
        ctx: Context<SetTicketAuthority>,
        new_authority_ticket_vault_bump: u8,
    ) -> Result<()> {
        instructions::tickets::set_ticket_authority::handle(ctx, new_authority_ticket_vault_bump)
    }
}
