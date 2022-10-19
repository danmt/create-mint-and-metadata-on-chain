use anchor_lang::prelude::*;

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