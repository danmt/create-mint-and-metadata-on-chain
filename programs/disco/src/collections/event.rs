use anchor_lang::prelude::*;

#[account]
pub struct Event {
    pub accepted_mint: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
    pub event_vault_bump: u8,
    pub event_mint_bump: u8,
    pub event_metadata_bump: u8,
    pub event_master_edition_bump: u8,
 /*
 pub name: String,                         // (40 + 4)
  pub description: String,                  // (500 + 4)
  pub banner: String,                       // (40 + 4)
  pub location: String,                     // (40 + 4)
  pub event_start_date: i64,                // 16
  pub event_end_date: i64,                  // 16
  pub event_id: u64,                        // 8
 */ 
}

impl Event {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 1 + 1 + 1 + 1;
}