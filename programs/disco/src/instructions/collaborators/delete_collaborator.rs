use {
    anchor_lang::prelude::*,
    crate::collections::{Collaborator, Event},
    crate::errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(event_id: String)]
pub struct DeleteCollaborator<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
            event_id.as_bytes()
        ],
        bump = event.bump,
        constraint = event.authority == authority.key() @ ErrorCode::OnlyEventAuthorityCanDeleteCollaborators
    )]
    pub event: Account<'info, Event>,
    /// CHECK: This account is used only as a base for derivation
    pub collaborator_base: UncheckedAccount<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [
            b"collaborator".as_ref(),
            event.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump
    )]
    pub collaborator: Account<'info, Collaborator>,
}

pub fn handle(
    _ctx: Context<DeleteCollaborator>,
  ) -> Result<()> {
    msg!("Deleting Collaborator...");
    Ok(())
  }