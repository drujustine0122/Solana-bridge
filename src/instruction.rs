#![allow(clippy::too_many_arguments)]
//! Instruction types

use std::mem::size_of;

use primitive_types::U256;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::BridgeInstruction::{
        Initialize, 
        UpdateCrogeProgram, 
        ExcludeFromFees,
        SetBridgeFee,
        ChangeGovernor,
        GetBridgeFee,
        SetBridgeFeesAddress,
        SetSystem,
        SetProcessedFess,
        GetProcessedFees,
        GetBridgeStatus,
        UpdateBridgingStaus,
        Swap,
        FeeCalculation,
        SwapBack,

    },
    state::{Bridge, BridgeConfig},
};


#[repr(C)]
#[derive(Clone, Copy)]
pub struct SwapPayload {
    pub amount: U256,
    pub toChainID: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SwapBackPayload {
    pub to: Pubkey,
    pub amount: U256,
    pub nonce: U256,
    pub fromChainID: u32,
}

/// Instructions supported by the SwapInfo program.
#[repr(C)]
pub enum BridgeInstruction {
    Initialize(),
    UpdateCrogeProgram(Pubkey),
    ExcludeFromFees(Pubkey, bool),
    SetBridgeFee(U256),
    ChangeGovernor(Pubkey),
    GetBridgeFee(),
    SetBridgeFeesAddress(U256),
    SetSystem(Pubkey),
    SetProcessedFess(u32, U256),
    GetProcessedFees(u32),
    GetBridgeStatus(U256, u32),
    UpdateBridgingStaus(bool),
    Swap(SwapPayload),
    FeeCalculation(U256),
    SwapBack(SwapBackPayload),
}



impl BridgeInstruction {
    /// Deserializes a byte buffer into a BridgeInstruction
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < size_of::<u8>() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(match input[0] {
            0 =>  Initialize(),
            1 => {
                let payload: &SwapPayload = unpack(input)?;

                Swap(*payload)
            }
            2 => {
                let payload: &SwapBackPayload = unpack(input)?;

                SwapBack(*payload)
            }
            3 => {
                let payload: &Pubkey = unpack(input)?;

                UpdateCrogeProgram(*payload)
            }
            4 => {
                let payload: &Pubkey = unpack(input)?;

                UpdateCrogeProgram(*payload)
            }
            6 => {
                let payload: &U256 = unpack(input)?;

                SetBridgeFee(*payload)
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    /// Serializes a BridgeInstruction into a byte buffer.
    pub fn serialize(self: Self) -> Result<Vec<u8>, ProgramError> {
        let mut output = Vec::with_capacity(size_of::<BridgeInstruction>());

        match self {
            Self::Initialize() => {
                output.resize(1, 0);
                output[0] = 0;
                #[allow(clippy::cast_ptr_alignment)]
                    let value = unsafe {
                    &mut *(&mut output[size_of::<u8>()] as *mut u8 as *mut InitializePayload)
                };
            }

            Self::Swap(payload) => {
                output.resize( 1, 0);
                output[0] = 1;

                if Bridge::_isBridgingPaused == true {
                    return Err("Contract is paused now");
                }

                let amount: U256 = payload.amount;
                let toChainID: u32 = payload.toChainID;

                let sender = Bridge::token_account_deserialize(sender_account_info)?;
                let payment = Bridge::token_account_deserialize(payment_info)?;

                if payment < Bridge::_processedFees[toChainID] {
                   return Err("Insufficient processed fees");
                }

                let _nonce: U256 = Bridge::_nonces[toChainID];
                _nonce = _nonce.add(1);
                Bridge::_nonce[toChainID] = _nonce;
                         

                let sender_account_info = next_account_info(account_info_iter)?;
                let bridge_info = Self::next_account_info_with_owner(account_info_iter, program_id)?;
                let transfer_info = next_account_info(account_info_iter)?;
                let payer_info = next_account_info(account_info_iter)?;

                
                let bridge_data = bridge_info.try_borrow_data()?;
                let bridge: &Bridge = Self::unpack_immutable(&bridge_data)?;
                let clock = Clock::from_account_info(clock_info)?;

                if *instructions_info.key != solana_program::sysvar::instructions::id() {
                    return Err(Error::InvalidSysvar.into());
                }
            }

            Self::SwapBack(payload) => {
                output.resize( 1, 0);
                output[0] = 1;
                let sender = Bridge::token_account_deserialize(sender_account_info)?;

                if sender != Bridge::config::system {
                    return Err("The caller is not system");
                }

                let to: Pubkey = payload.to;
                let amount: U256 = payload.amount;
                let nonce: U256 = payload.nonce;
                let fromChainID: u32 = payload.fromChainID;

                if Bridge::nonceProcessed[fromChainID][nonce] == true {
                    return Err("Swap is already proceeds");
                }

                Bridge::nonceProcessed[fromChainID][nonce] = true;

                let mut temp: U256;
                if Bridge::_isExcludedFromFees[to] == true {
                    temp = amount;
                } else {
                    temp = Bridge::feeCalculation(amount);
                }
                let fees: U256 = amount.sub(temp);

                if fees > 0 {
                    Self::Transfer({
                        token: Bridge::config::croge_program,
                        to: Bridge::config::bridgeFeesAddress,
                        amount: fees
                    })
                } 

                Self::Transfer({
                    token: Bridge::config::croge_program,
                    to: to,
                    amount: temp
                })
        
                Ok(())
            }

            Self::Transfer(payload) => {
                output.resize(size_of::<TransferOutPayloadRaw>() + 1, 0);
                output[0] = 1;
                #[allow(clippy::cast_ptr_alignment)]
                    let value = unsafe {
                    &mut *(&mut output[size_of::<u8>()] as *mut u8 as *mut TransferOutPayloadRaw)
                };

                let mut amount_bytes = [0u8; 32];
                payload.amount.to_big_endian(&mut amount_bytes);

                *value = TransferOutPayloadRaw {
                    amount: amount_bytes,
                    chain_id: payload.chain_id,
                    asset: payload.asset,
                    target: payload.target,
                    nonce: payload.nonce,
                };
            }
           
            
            Self::CreateWrapped(payload) => {
                output.resize(size_of::<AssetMeta>() + 1, 0);
                output[0] = 7;
                #[allow(clippy::cast_ptr_alignment)]
                    let value =
                    unsafe { &mut *(&mut output[size_of::<u8>()] as *mut u8 as *mut AssetMeta) };
                *value = payload;
            }
        }
        Ok(output)
    }
}

/// Creates an 'initialize' instruction.
#[cfg(not(target_arch = "bpf"))]
pub fn initialize(
    program_id: &Pubkey,
    sender: &Pubkey,
    initial_guardian: Vec<[u8; 20]>,
    config: &BridgeConfig,
) -> Result<Instruction, ProgramError> {
    if initial_guardian.len() > MAX_LEN_GUARDIAN_KEYS {
        return Err(ProgramError::InvalidArgument);
    }
    let mut initial_g = [[0u8; 20]; MAX_LEN_GUARDIAN_KEYS];
    for (i, key) in initial_guardian.iter().enumerate() {
        initial_g[i] = *key;
    }
    let data = BridgeInstruction::Initialize(InitializePayload {
        config: *config,
        len_guardians: initial_guardian.len() as u8,
        initial_guardian: initial_g,
    })
        .serialize()?;

    let bridge_key = Bridge::derive_bridge_id(program_id)?;
    let guardian_set_key = Bridge::derive_guardian_set_id(program_id, &bridge_key, 0)?;

    let accounts = vec![
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
        AccountMeta::new(bridge_key, false),
        AccountMeta::new(guardian_set_key, false),
        AccountMeta::new(*sender, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates an 'TransferOut' instruction.
#[cfg(not(target_arch = "bpf"))]
pub fn transfer_out(
    program_id: &Pubkey,
    payer: &Pubkey,
    token_account: &Pubkey,
    token_mint: &Pubkey,
    t: &TransferOutPayload,
) -> Result<Instruction, ProgramError> {
    let data = BridgeInstruction::TransferOut(*t).serialize()?;

    let bridge_key = Bridge::derive_bridge_id(program_id)?;
    let transfer_key = Bridge::derive_transfer_id(
        program_id,
        &bridge_key,
        t.asset.chain,
        t.asset.address,
        t.chain_id,
        t.target,
        token_account.to_bytes(),
        t.nonce,
    )?;

    let mut accounts = vec![
        AccountMeta::new_readonly(*program_id, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::instructions::id(), false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(bridge_key, false),
        AccountMeta::new(transfer_key, false),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new(*payer, true),
    ];

    // If the token is a native solana token add a custody account
    if t.asset.chain == CHAIN_ID_SOLANA {
        let custody_key = Bridge::derive_custody_id(program_id, &bridge_key, token_mint)?;
        accounts.push(AccountMeta::new(custody_key, false));
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

