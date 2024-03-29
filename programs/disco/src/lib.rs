use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, mint_to, transfer, Burn, Mint, MintTo, Token, TokenAccount, Transfer},
};

declare_id!("EJQnbXhsLS92wsAXg1vPaZt88hfzmuhcqBVLQBn9h23x");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(ctx: Context<CreateEvent>, event_title: String) -> Result<()> {
        (*ctx.accounts.event).event_title = event_title.clone();
        (*ctx.accounts.event).accepted_mint = ctx.accounts.accepted_mint.key();
        (*ctx.accounts.event).authority = ctx.accounts.authority.key();
        (*ctx.accounts.event).bump = *ctx.bumps.get("event").unwrap();
        (*ctx.accounts.event).event_vault_bump = *ctx.bumps.get("event_vault").unwrap();

        Ok(())
    }

    pub fn create_collaborator(ctx: Context<CreateCollaborator>) -> Result<()> {
        ctx.accounts.collaborator.bump = *ctx.bumps.get("collaborator").unwrap();

        Ok(())
    }

    pub fn delete_collaborator(_ctx: Context<DeleteCollaborator>) -> Result<()> {
        Ok(())
    }

    pub fn create_event_ticket(
        ctx: Context<CreateEventTicket>,
        ticket_name: String,
        ticket_symbol: String,
        ticket_uri: String,
        ticket_price: u32,
        ticket_quantity: u32,
    ) -> Result<()> {
        (*ctx.accounts.event_ticket).price = ticket_price;
        (*ctx.accounts.event_ticket).quantity = ticket_quantity;
        (*ctx.accounts.event_ticket).sold = 0;
        (*ctx.accounts.event_ticket).used = 0;
        (*ctx.accounts.event_ticket).bump = *ctx.bumps.get("event_ticket").unwrap();
        (*ctx.accounts.event_ticket).ticket_mint_bump = *ctx.bumps.get("ticket_mint").unwrap();
        (*ctx.accounts.event_ticket).ticket_metadata_bump =
            *ctx.bumps.get("ticket_metadata").unwrap();

        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        solana_program::program::invoke_signed(
            &mpl_token_metadata::instruction::create_metadata_accounts_v3(
                mpl_token_metadata::ID,
                (*ctx.accounts.ticket_metadata).key(),
                ctx.accounts.ticket_mint.key(),
                ctx.accounts.event.key(),
                (*ctx.accounts.authority).key(),
                ctx.accounts.event.key(),
                ticket_name,
                ticket_symbol,
                ticket_uri,
                None,
                0,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                ctx.accounts.metadata_program.to_account_info().clone(),
                ctx.accounts.ticket_metadata.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
                ctx.accounts.ticket_mint.to_account_info().clone(),
                ctx.accounts.event.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }

    pub fn create_proof_of_attendance(
        ctx: Context<CreateProofOfAttendance>,
        poap_name: String,
        poap_symbol: String,
        poap_uri: String,
    ) -> Result<()> {
        (*ctx.accounts.event_ticket).has_poap = true;
        (*ctx.accounts.event_ticket).attendance_mint_bump =
            *ctx.bumps.get("attendance_mint").unwrap();
        (*ctx.accounts.event_ticket).attendance_metadata_bump =
            *ctx.bumps.get("attendance_metadata").unwrap();

        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        solana_program::program::invoke_signed(
            &mpl_token_metadata::instruction::create_metadata_accounts_v3(
                mpl_token_metadata::ID,
                (*ctx.accounts.attendance_metadata).key(),
                ctx.accounts.attendance_mint.key(),
                ctx.accounts.event.key(),
                (*ctx.accounts.authority).key(),
                ctx.accounts.event.key(),
                poap_name,
                poap_symbol,
                poap_uri,
                None,
                0,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                ctx.accounts.metadata_program.to_account_info().clone(),
                ctx.accounts.attendance_metadata.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
                ctx.accounts.attendance_mint.to_account_info().clone(),
                ctx.accounts.event.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }

    pub fn buy_tickets(ctx: Context<BuyTickets>, ticket_quantity: u32) -> Result<()> {
        (*ctx.accounts.event_ticket).sold += ticket_quantity;

        // call transfer from authority to event vault
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_vault.to_account_info(),
                    to: ctx.accounts.event_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            ctx.accounts
                .event_ticket
                .price
                .checked_mul(ticket_quantity.into())
                .unwrap()
                .into(),
        )?;

        // call mintTo instruction
        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    to: ctx.accounts.ticket_vault.to_account_info(),
                    authority: ctx.accounts.event.to_account_info(),
                },
                &[&seeds[..]],
            ),
            ticket_quantity.into(),
        )?;

        Ok(())
    }

    pub fn check_in(ctx: Context<CheckIn>, ticket_quantity: u32) -> Result<()> {
        if ctx.accounts.event_ticket.has_poap {
            return Err(ErrorCode::CheckInIsOnlyAvailableForTicketsWithoutAttendance.into());
        }

        (*ctx.accounts.event_ticket).used += ticket_quantity;

        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    authority: ctx.accounts.attendee.to_account_info(),
                    from: ctx.accounts.ticket_vault.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                },
            ),
            ticket_quantity.into(),
        )?;

        Ok(())
    }

    pub fn check_in_with_attendance(
        ctx: Context<CheckInWithAttendance>,
        ticket_quantity: u32,
    ) -> Result<()> {
        (*ctx.accounts.event_ticket).used += ticket_quantity;

        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    authority: ctx.accounts.attendee.to_account_info(),
                    from: ctx.accounts.ticket_vault.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                },
            ),
            ticket_quantity.into(),
        )?;

        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.attendance_mint.to_account_info(),
                    to: ctx.accounts.attendance_vault.to_account_info(),
                    authority: ctx.accounts.event.to_account_info(),
                },
                &[&seeds[..]],
            ),
            ticket_quantity.into(),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(event_title: String)]
pub struct CreateEvent<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = Event::SIZE,
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump
    )]
    pub event: Account<'info, Event>,
    pub accepted_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::authority = event,
        token::mint = accepted_mint,
        seeds = [
            b"event_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump
    )]
    pub event_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct CreateCollaborator<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump,
        constraint = event.authority == authority.key() @ ErrorCode::OnlyEventAuthorityCanCreateCollaborators
    )]
    pub event: Account<'info, Event>,
    /// CHECK: This account is used only as a base for derivation
    pub collaborator_base: UncheckedAccount<'info>,
    /// CHECK: This account is created in this instruction
    #[account(
        init,
        space = Collaborator::SIZE,
        payer = authority,
        seeds = [
            b"collaborator".as_ref(),
            event.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump
    )]
    pub collaborator: Account<'info, Collaborator>,
}

#[derive(Accounts)]
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

#[derive(Accounts)]
#[instruction(
    ticket_name: String,
    ticket_symbol: String,
    ticket_uri: String,
    ticket_price: u32,
    ticket_quantity: u32,
)]
pub struct CreateEventTicket<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    /// CHECK: This is used only for generating the PDA.
    pub event_ticket_base: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = EventTicket::SIZE,
        seeds = [
            b"event_ticket".as_ref(),
            event.key().as_ref(),
            event_ticket_base.key().as_ref(),
        ],
        bump
    )]
    pub event_ticket: Account<'info, EventTicket>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = event,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub ticket_metadata: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(
    poap_name: String,
    poap_symbol: String,
    poap_uri: String,
)]
pub struct CreateProofOfAttendance<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    /// CHECK: This is used only for generating the PDA.
    pub event_ticket_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"event_ticket".as_ref(),
            event.key().as_ref(),
            event_ticket_base.key().as_ref(),
        ],
        bump = event_ticket.bump
    )]
    pub event_ticket: Account<'info, EventTicket>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = event,
        seeds = [
            b"attendance_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump
    )]
    pub attendance_mint: Account<'info, Mint>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            attendance_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub attendance_metadata: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(ticket_quantity: u32)]
pub struct BuyTickets<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    /// CHECK: This is used only for generating the PDA.
    pub event_ticket_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"event_ticket".as_ref(),
            event.key().as_ref(),
            event_ticket_base.key().as_ref(),
        ],
        bump = event_ticket.bump,
        constraint = event_ticket.quantity >= event_ticket.sold + ticket_quantity @ ErrorCode::NotEnoughTicketsAvailable
    )]
    pub event_ticket: Account<'info, EventTicket>,
    #[account(
        mut,
        constraint = buyer_vault.mint == event.accepted_mint
    )]
    pub buyer_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"event_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump = event.event_vault_bump
    )]
    pub event_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump = event_ticket.ticket_mint_bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = ticket_vault.mint == ticket_mint.key()
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
#[instruction(ticket_quantity: u32)]
pub struct CheckIn<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub collaborator_base: Signer<'info>,
    pub attendee: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    #[account(
        seeds = [
            b"collaborator".as_ref(),
            event.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump
    )]
    pub collaborator: Account<'info, Collaborator>,
    /// CHECK: This is used only for generating the PDA.
    pub event_ticket_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"event_ticket".as_ref(),
            event.key().as_ref(),
            event_ticket_base.key().as_ref(),
        ],
        bump = event_ticket.bump,
        constraint = event_ticket.sold - event_ticket.used >= ticket_quantity @ ErrorCode::NotEnoughTicketsToCheckIn
    )]
    pub event_ticket: Account<'info, EventTicket>,
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump = event_ticket.ticket_mint_bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = ticket_vault.mint == ticket_mint.key()
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
#[instruction(ticket_quantity: u32)]
pub struct CheckInWithAttendance<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub collaborator_base: Signer<'info>,
    pub attendee: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
        ],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,
    #[account(
        seeds = [
            b"collaborator".as_ref(),
            event.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump
    )]
    pub collaborator: Account<'info, Collaborator>,
    /// CHECK: This is used only for generating the PDA.
    pub event_ticket_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"event_ticket".as_ref(),
            event.key().as_ref(),
            event_ticket_base.key().as_ref(),
        ],
        bump = event_ticket.bump,
        constraint = event_ticket.sold - event_ticket.used >= ticket_quantity @ ErrorCode::NotEnoughTicketsToCheckIn
    )]
    pub event_ticket: Account<'info, EventTicket>,
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump = event_ticket.ticket_mint_bump
    )]
    pub ticket_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = ticket_vault.mint == ticket_mint.key()
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            b"attendance_mint".as_ref(),
            event.key().as_ref(),
            event_ticket.key().as_ref(),
        ],
        bump = event_ticket.attendance_mint_bump
    )]
    pub attendance_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = attendance_vault.mint == attendance_mint.key()
    )]
    pub attendance_vault: Box<Account<'info, TokenAccount>>,
}

#[account]
pub struct Event {
    pub event_title: String, // max 32
    pub accepted_mint: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
    pub event_vault_bump: u8,
}

impl Event {
    pub const SIZE: usize = 8 + 36 + 32 + 32 + 1 + 1;
}

#[account]
pub struct Collaborator {
    pub bump: u8,
}

impl Collaborator {
    pub const SIZE: usize = 8 + 1;
}

#[account]
pub struct EventTicket {
    pub price: u32,
    pub quantity: u32,
    pub sold: u32,
    pub used: u32,
    pub has_poap: bool,
    pub bump: u8,
    pub ticket_mint_bump: u8,
    pub ticket_metadata_bump: u8,
    pub attendance_mint_bump: u8,
    pub attendance_metadata_bump: u8,
}

impl EventTicket {
    pub const SIZE: usize = 8 + 4 + 4 + 4 + 4 + 1 + 1 + 1 + 1 + 1 + 1;
}

#[error_code]
pub enum ErrorCode {
    #[msg("There are not enough tickets available.")]
    NotEnoughTicketsAvailable,
    #[msg("There are not enough tickets to check-in.")]
    NotEnoughTicketsToCheckIn,
    #[msg("Only event authority can create collaborators.")]
    OnlyEventAuthorityCanCreateCollaborators,
    #[msg("Only event authority can delete collaborators.")]
    OnlyEventAuthorityCanDeleteCollaborators,
    #[msg("Check in instruction is only available for tickets without attendance.")]
    CheckInIsOnlyAvailableForTicketsWithoutAttendance,
}
