//! Program entrypoint definitions
#![cfg(feature = "program")]
#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{
    account_info::AccountInfo,entrypoint, entrypoint::ProgramResult, program_error::PrintProgramError,
    pubkey::Pubkey,
};

use crate::{error::Error, state::Bridge};

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("In bridge entrypoint");
    if let Err(error) = Bridge::process(program_id, accounts, instruction_data) {
        error.print::<Error>();
        return Err(error);
    }
    Ok(())
}
