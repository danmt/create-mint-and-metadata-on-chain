use anchor_lang::prelude::*;

#[account]
pub struct Collaborator {
    pub bump: u8,
}

impl Collaborator {
    pub const SIZE: usize = 8 + 1;
}