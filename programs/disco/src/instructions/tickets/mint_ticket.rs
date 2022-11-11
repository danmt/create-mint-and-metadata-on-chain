use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint,Token, TokenAccount},
    },
    anchor_spl::{
        token::{mint_to, transfer,MintTo,  Transfer},
    },
    crate::collections::{Event,TicketMachine,Ticket},
    crate::errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(ticket_vault_bump: u8, event_id: String)]
pub struct MintTicket<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub event_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"event".as_ref(),
            event_base.key().as_ref(),
            event_id.as_bytes()
        ],
        bump = event.bump
    )]
    pub event: Box<Account<'info, Event>>,
    #[account(
        seeds = [
            b"event_mint".as_ref(),
            event.key().as_ref(),
        ],
        bump = event.event_mint_bump
    )]
    pub event_mint: Account<'info, Mint>,
    /// CHECK: This will be verified by token metadata program.
    #[account(
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            event_mint.key().as_ref(),
        ],
        bump = event.event_metadata_bump,
        seeds::program = metadata_program.key()
    )]
    pub event_metadata: UncheckedAccount<'info>,
    /// CHECK: This will be verified by token metadata program.
    #[account(
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            event_mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump = event.event_master_edition_bump,
        seeds::program = metadata_program.key()
    )]
    pub event_master_edition: UncheckedAccount<'info>,
    /// CHECK: This is used only for generating the PDA.
    pub ticket_machine_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_base.key().as_ref(),
        ],
        bump = ticket_machine.bump,
        constraint = ticket_machine.quantity >= ticket_machine.sold + 1 @ ErrorCode::NotEnoughTicketsAvailable
    )]
    pub ticket_machine: Box<Account<'info, TicketMachine>>,
    #[account(
        mut,
        constraint = buyer_vault.mint == event.accepted_mint
    )]
    pub buyer_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            b"event_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump = event.event_vault_bump
    )]
    pub event_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is used only for generating the PDA.
    pub ticket_mint_base: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = event,
        mint::freeze_authority = event,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            ticket_machine.key().as_ref(),
            ticket_mint_base.key().as_ref()
        ],
        bump
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
        bump,
        seeds::program = metadata_program.key()
    )]
    pub ticket_metadata: UncheckedAccount<'info>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub ticket_master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::authority = authority,
        associated_token::mint = ticket_mint,
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        space = Ticket::SIZE,
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump,
    )]
    pub ticket: Box<Account<'info, Ticket>>,
}

pub fn handle(ctx: Context<MintTicket>, ticket_vault_bump: u8) -> Result<()> {
    (*ctx.accounts.ticket_machine).sold += 1;
    (*ctx.accounts.ticket).authority = ctx.accounts.authority.key();
    (*ctx.accounts.ticket).checked_in = false;
    (*ctx.accounts.ticket).bump = *ctx.bumps.get("ticket").unwrap();
    (*ctx.accounts.ticket).associated_token_bump = ticket_vault_bump;
    (*ctx.accounts.ticket).mint_bump = *ctx.bumps.get("ticket_mint").unwrap();
    (*ctx.accounts.ticket).metadata_bump = *ctx.bumps.get("ticket_metadata").unwrap();
    (*ctx.accounts.ticket).master_edition_bump =
        *ctx.bumps.get("ticket_master_edition").unwrap();

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
        ctx.accounts.ticket_machine.price,
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
        1,
    )?;

    solana_program::program::invoke_signed(
        &mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata::ID,
            (*ctx.accounts.ticket_metadata).key(),
            (*ctx.accounts.ticket_mint).key(),
            (*ctx.accounts.event).key(),
            (*ctx.accounts.authority).key(),
            (*ctx.accounts.event).key(),
            (*ctx.accounts.ticket_machine).name.clone(),
            (*ctx.accounts.ticket_machine).symbol.clone(),
            (*ctx.accounts.ticket_machine).uri.clone(),
            None,
            0,
            true,
            true,
            None,
            Some(mpl_token_metadata::state::Uses {
                remaining: (*ctx.accounts.ticket_machine).uses,
                total: (*ctx.accounts.ticket_machine).uses,
                use_method: match (*ctx.accounts.ticket_machine).uses {
                    1 => mpl_token_metadata::state::UseMethod::Single,
                    _ => mpl_token_metadata::state::UseMethod::Multiple,
                },
            }),
            None,
        ),
        &[
            ctx.accounts.ticket_metadata.to_account_info().clone(),
            ctx.accounts.ticket_mint.to_account_info().clone(),
            ctx.accounts.event.to_account_info().clone(),
            ctx.accounts.authority.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone(),
        ],
        &[&seeds[..]],
    )?;

    solana_program::program::invoke_signed(
        &mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata::ID,
            (*ctx.accounts.ticket_master_edition).key(),
            ctx.accounts.ticket_mint.key(),
            ctx.accounts.event.key(),
            ctx.accounts.event.key(),
            ctx.accounts.ticket_metadata.key(),
            (*ctx.accounts.authority).key(),
            Some(0),
        ),
        &[
            ctx.accounts.ticket_master_edition.to_account_info().clone(),
            ctx.accounts.ticket_mint.to_account_info().clone(),
            ctx.accounts.event.to_account_info().clone(),
            ctx.accounts.authority.to_account_info().clone(),
            ctx.accounts.ticket_metadata.to_account_info().clone(),
            ctx.accounts.token_program.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone(),
        ],
        &[&seeds[..]],
    )?;

    solana_program::program::invoke_signed(
        &mpl_token_metadata::instruction::set_and_verify_collection(
            mpl_token_metadata::ID,
            (*ctx.accounts.ticket_metadata).key(),
            (*ctx.accounts.event).key(),
            (*ctx.accounts.authority).key(),
            (*ctx.accounts.event).key(),
            ctx.accounts.event_mint.key(),
            (*ctx.accounts.event_metadata).key(),
            (*ctx.accounts.event_master_edition).key(),
            None,
        ),
        &[
            ctx.accounts.ticket_metadata.to_account_info().clone(),
            ctx.accounts.event.to_account_info().clone(),
            ctx.accounts.authority.to_account_info().clone(),
            ctx.accounts.event_mint.to_account_info().clone(),
            ctx.accounts.event_metadata.to_account_info().clone(),
            ctx.accounts.event_master_edition.to_account_info().clone(),
        ],
        &[&seeds[..]],
    )?;

    Ok(())
}