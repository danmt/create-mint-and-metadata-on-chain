use anchor_lang::prelude::*;

#[account]
pub struct TicketMachine {
    pub name: String,   // 32
    pub symbol: String, // 10
    pub uri: String,    // 200
    pub price: u64,
    pub quantity: u64,
    pub sold: u64,
    pub used: u64,
    pub uses: u64,
    pub bump: u8,
}

impl TicketMachine {
    pub const SIZE: usize = 8 + 36 + 204 + 14 + 8 + 8 + 8 + 8 + 8 + 1;
}
