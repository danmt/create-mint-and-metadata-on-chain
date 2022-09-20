use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};

declare_id!("EJQnbXhsLS92wsAXg1vPaZt88hfzmuhcqBVLQBn9h23x");

#[program]
pub mod disco {
    use super::*;

    pub fn create_event(
        ctx: Context<CreateEvent>,
        event_name: String,
        event_symbol: String,
        event_uri: String,
    ) -> Result<()> {
        (*ctx.accounts.event).accepted_mint = ctx.accounts.accepted_mint.key();
        (*ctx.accounts.event).authority = ctx.accounts.authority.key();
        (*ctx.accounts.event).bump = *ctx.bumps.get("event").unwrap();
        (*ctx.accounts.event).event_vault_bump = *ctx.bumps.get("event_vault").unwrap();
        (*ctx.accounts.event).event_mint_bump = *ctx.bumps.get("event_mint").unwrap();
        (*ctx.accounts.event).event_metadata_bump = *ctx.bumps.get("event_metadata").unwrap();
        (*ctx.accounts.event).event_master_edition_bump =
            *ctx.bumps.get("event_master_edition").unwrap();

        let seeds = &[
            b"event".as_ref(),
            ctx.accounts.event_base.to_account_info().key.as_ref(),
            &[ctx.accounts.event.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.event_mint.to_account_info(),
                    to: ctx.accounts.event_collection_vault.to_account_info(),
                    authority: ctx.accounts.event.to_account_info(),
                },
                &[&seeds[..]],
            ),
            1,
        )?;

        solana_program::program::invoke_signed(
            &mpl_token_metadata::instruction::create_metadata_accounts_v3(
                mpl_token_metadata::ID,
                (*ctx.accounts.event_metadata).key(),
                ctx.accounts.event_mint.key(),
                ctx.accounts.event.key(),
                (*ctx.accounts.authority).key(),
                ctx.accounts.event.key(),
                event_name,
                event_symbol,
                event_uri,
                None,
                0,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                ctx.accounts.event_metadata.to_account_info().clone(),
                ctx.accounts.event_mint.to_account_info().clone(),
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
                (*ctx.accounts.event_master_edition).key(),
                ctx.accounts.event_mint.key(),
                ctx.accounts.event.key(),
                ctx.accounts.event.key(),
                ctx.accounts.event_metadata.key(),
                (*ctx.accounts.authority).key(),
                Some(0),
            ),
            &[
                ctx.accounts.event_master_edition.to_account_info().clone(),
                ctx.accounts.event_mint.to_account_info().clone(),
                ctx.accounts.event.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.event_metadata.to_account_info().clone(),
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }

    pub fn create_collaborator(ctx: Context<CreateCollaborator>) -> Result<()> {
        ctx.accounts.collaborator.bump = *ctx.bumps.get("collaborator").unwrap();

        Ok(())
    }

    pub fn delete_collaborator(_ctx: Context<DeleteCollaborator>) -> Result<()> {
        Ok(())
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

    pub fn mint_ticket(ctx: Context<MintTicket>, ticket_vault_bump: u8) -> Result<()> {
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

    pub fn check_in(ctx: Context<CheckIn>) -> Result<()> {
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

    pub fn verify_ticket_ownership(_ctx: Context<VerifyTicketOwnership>) -> Result<()> {
        Ok(())
    }

    pub fn set_ticket_authority(
        ctx: Context<SetTicketAuthority>,
        new_authority_ticket_vault_bump: u8,
    ) -> Result<()> {
        (*ctx.accounts.ticket).authority = ctx.accounts.new_authority.key();
        (*ctx.accounts.ticket).associated_token_bump = new_authority_ticket_vault_bump;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(event_name: String, event_symbol: String, event_uri: String)]
pub struct CreateEvent<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is being validated.
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
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
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = event,
        seeds = [
            b"event_mint".as_ref(),
            event.key().as_ref(),
        ],
        bump
    )]
    pub event_mint: Account<'info, Mint>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            event_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub event_metadata: UncheckedAccount<'info>,
    /// CHECK: This will be verified by token metadata program.
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            event_mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub event_master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        token::authority = event,
        token::mint = event_mint,
        seeds = [
            b"event_collection_vault".as_ref(),
            event.key().as_ref(),
        ],
        bump
    )]
    pub event_collection_vault: Account<'info, TokenAccount>,
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
    ticket_price: u64,
    ticket_quantity: u64,
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
    pub ticket_machine_base: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = TicketMachine::SIZE,
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_base.key().as_ref(),
        ],
        bump
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
}

#[derive(Accounts)]
#[instruction(ticket_vault_bump: u8)]
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

#[derive(Accounts)]
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
    pub ticket_machine_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_base.key().as_ref(),
        ],
        bump = ticket_machine.bump,
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
    /// CHECK: this is only used to generate a PDA
    pub ticket_mint_base: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            ticket_machine.key().as_ref(),
            ticket_mint_base.key().as_ref()
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

#[derive(Accounts)]
pub struct VerifyTicketOwnership<'info> {
    /// CHECK: this is verified through an address constraint
    #[account(address = mpl_token_metadata::ID, executable)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub collaborator_base: Signer<'info>,
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
    #[account(
        seeds = [
            b"collaborator".as_ref(),
            event.key().as_ref(),
            collaborator_base.key().as_ref(),
        ],
        bump = collaborator.bump
    )]
    pub collaborator: Account<'info, Collaborator>,
    /// CHECK: This is used only for generating the PDA.
    pub ticket_machine_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"ticket_machine".as_ref(),
            event.key().as_ref(),
            ticket_machine_base.key().as_ref(),
        ],
        bump = ticket_machine.bump,
    )]
    pub ticket_machine: Account<'info, TicketMachine>,
    /// CHECK: this is only used to generate a PDA
    pub ticket_mint_base: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"ticket_mint".as_ref(),
            event.key().as_ref(),
            ticket_machine.key().as_ref(),
            ticket_mint_base.key().as_ref()
        ],
        bump = ticket.mint_bump
    )]
    pub ticket_mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.bump,
        constraint = ticket.authority == authority.key() @ ErrorCode::InvalidAuthorityForTicket
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    #[account(
        seeds = [
            authority.key().as_ref(),
            token_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.associated_token_bump,
        seeds::program = associated_token_program.key(),
        constraint = ticket_vault.amount > 0 @ ErrorCode::InvalidAuthorityForTicket,
    )]
    pub ticket_vault: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
#[instruction(new_authority_ticket_vault_bump: u8)]
pub struct SetTicketAuthority<'info> {
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub authority: Signer<'info>,
    /// CHECK: new authority can be anything.
    pub new_authority: UncheckedAccount<'info>,
    pub ticket_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"ticket".as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = ticket.bump,
        constraint = ticket.authority == authority.key() @ ErrorCode::OnlyTicketAuthorityCanChangeAuthority,
        constraint = !ticket.checked_in @ ErrorCode::CheckedInTicketsCantChangeAuthority,
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    #[account(
        seeds = [
            new_authority.key().as_ref(),
            token_program.key().as_ref(),
            ticket_mint.key().as_ref(),
        ],
        bump = new_authority_ticket_vault_bump,
        seeds::program = associated_token_program.key()
    )]
    pub new_authority_ticket_vault: Box<Account<'info, TokenAccount>>,
}

#[account]
pub struct Event {
    pub accepted_mint: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
    pub event_vault_bump: u8,
    pub event_mint_bump: u8,
    pub event_metadata_bump: u8,
    pub event_master_edition_bump: u8,
}

impl Event {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 1 + 1 + 1 + 1;
}

#[account]
pub struct Collaborator {
    pub bump: u8,
}

impl Collaborator {
    pub const SIZE: usize = 8 + 1;
}

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

#[error_code]
pub enum ErrorCode {
    #[msg("There are not enough tickets available.")]
    NotEnoughTicketsAvailable,
    #[msg("This ticket has already checked-in.")]
    TicketAlreadyCheckedIn,
    #[msg("Only event authority can create collaborators.")]
    OnlyEventAuthorityCanCreateCollaborators,
    #[msg("Only event authority can delete collaborators.")]
    OnlyEventAuthorityCanDeleteCollaborators,
    #[msg("The authority is not registered as the authority of the ticket.")]
    InvalidAuthorityForTicket,
    #[msg("The only the authority of the ticket can set a new authority.")]
    OnlyTicketAuthorityCanChangeAuthority,
    #[msg("Ticket that have already been checked in can't change authority.")]
    CheckedInTicketsCantChangeAuthority,
}
