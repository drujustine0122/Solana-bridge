//! Bridge transition types

use std::{collections::HashMap};

use primitive_types::U256;
use solana_program::{pubkey::Pubkey};


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BridgeConfig {
    pub croge_program: Pubkey,
    pub system: Pubkey,
    pub governor: Pubkey,
    pub bridgeFeesAddress: Pubkey,
    pub owner: Pubkey,
    pub _bridgeFee: u32,
 }

/// Bridge state.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Bridge {
    pub _nonces: HashMap<U256, U256>,
    pub nonceProcessed: HashMap<u32, HashMap<U256, bool>>,
    pub _processedFees: HashMap<u32, U256>,
    pub _isExcludedFromFees: HashMap<Pubkey, bool>,
    pub _isBridgingPaused: bool,

    pub config: BridgeConfig,
}

