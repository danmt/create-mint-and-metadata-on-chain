use {
    anchor_lang::prelude::*, 
    anchor_spl::token::*,
    crate::collections::Event 
};


#[derive(Accounts)]
#[instruction(event_name: String, event_symbol: String, event_uri: String, event_id: String)]
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
            event_id.as_bytes()
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
        mint::freeze_authority = event,
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

pub fn handle(
    ctx: Context<CreateEvent>,
    event_name: String,
    event_symbol: String,
    event_uri: String,
    event_id: String
  ) -> Result<()> {
    msg!("Creating Event...");
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
        event_id.as_bytes(),
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
            mpl_token_metadata::ID, //program_id
            (*ctx.accounts.event_metadata).key(), //metadata_account
            ctx.accounts.event_mint.key(), //mint
            ctx.accounts.event.key(), //mint_authority
            (*ctx.accounts.authority).key(), //payer
            ctx.accounts.event.key(), //update_authority
            event_name, //name
            event_symbol, //symbol
            event_uri, //uri
            None, //creators
            0, //seller_fee_basis_points
            true, //update_authority_is_signer
            true, //is_mutable
            None, //collection
            None, //uses
            None, //collection_details
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
            mpl_token_metadata::ID, //program_id
            (*ctx.accounts.event_master_edition).key(), //edition
            ctx.accounts.event_mint.key(), //mint
            ctx.accounts.event.key(), //update_authority
            ctx.accounts.event.key(), //mint_authority
            ctx.accounts.event_metadata.key(), //metadata
            (*ctx.accounts.authority).key(), //payer
            Some(0), //max_supply
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
  