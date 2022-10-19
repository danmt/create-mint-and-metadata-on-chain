use anchor_lang::prelude::*;

#[account]
pub struct Ticket {
    pub authority: Pubkey,
    pub checked_in: bool,
    pub bump: u8,
    pub associated_token_bump: u8,
    pub mint_bump: u8,
    pub metadata_bump: u8,
    pub master_edition_bump: u8,
}

impl Ticket {
    pub const SIZE: usize = 8 + 32 + 1 + 1 + 1 + 1 + 1 + 1;
}