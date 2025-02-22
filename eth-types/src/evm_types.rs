//! Evm types needed for parsing instruction sets as well

use serde::{Deserialize, Serialize};
use std::fmt;

pub mod memory;
pub mod opcode_ids;
pub mod stack;
pub mod storage;

pub use {
    memory::{Memory, MemoryAddress},
    opcode_ids::OpcodeId,
    stack::{Stack, StackAddress},
    storage::Storage,
};

/// Wrapper type over `usize` which represents the program counter of the Evm.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ProgramCounter(pub usize);

impl fmt::Debug for ProgramCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:06x}", self.0))
    }
}

impl From<ProgramCounter> for usize {
    fn from(addr: ProgramCounter) -> usize {
        addr.0
    }
}

impl From<usize> for ProgramCounter {
    fn from(pc: usize) -> Self {
        ProgramCounter(pc)
    }
}

impl ProgramCounter {
    /// Increase Self by one
    pub fn inc(&mut self) {
        self.0 += 1;
    }

    /// Increase Self by one and return the value before the increase.
    pub fn inc_pre(&mut self) -> Self {
        let pre = *self;
        self.inc();
        pre
    }
}

/// Defines the gas left to perate.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Gas(pub u64);

impl fmt::Debug for Gas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

/// Quotient for max refund of gas used
pub const MAX_REFUND_QUOTIENT_OF_GAS_USED: usize = 5;

/// Defines the gas consumption.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GasCost(pub u64);

impl fmt::Debug for GasCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl GasCost {
    /// Constant cost for free step
    pub const ZERO: Self = Self(0);
    /// Constant cost for jumpdest step, only takes one gas
    pub const ONE: Self = Self(1);
    /// Constant cost for quick step
    pub const QUICK: Self = Self(2);
    /// Constant cost for fastest step
    pub const FASTEST: Self = Self(3);
    /// Constant cost for fast step
    pub const FAST: Self = Self(5);
    /// Constant cost for mid step
    pub const MID: Self = Self(8);
    /// Constant cost for slow step
    pub const SLOW: Self = Self(10);
    /// Constant cost for ext step
    pub const EXT: Self = Self(20);
    /// Constant cost for SHA3
    pub const SHA3: Self = Self(30);
    /// Constant cost for SELFDESTRUCT
    pub const SELFDESTRUCT: Self = Self(5000);
    /// Constant cost for CREATE
    pub const CREATE: Self = Self(32000);
    /// Constant cost for every additional word when expanding memory
    pub const MEMORY: Self = Self(3);
    /// Constant cost for copying every word
    pub const COPY: Self = Self(3);
    /// Constant cost for a cold SLOAD
    pub const COLD_SLOAD_COST: Self = Self(2100);
    /// Constant cost for a cold account access
    pub const COLD_ACCOUNT_ACCESS_COST: Self = Self(2600);
    /// Constant cost for a warm storage read
    pub const WARM_STORAGE_READ_COST: Self = Self(100);
    /// Constant cost for a basic storage operation
    pub const SLOAD_GAS: Self = Self(100);
    /// Constant cost for a storage set
    pub const SSTORE_SET_GAS: Self = Self(20000);
    /// Constant cost for a storage reset
    pub const SSTORE_RESET_GAS: Self = Self(2900);
    /// Constant cost for a storage clear
    pub const SSTORE_CLEARS_SCHEDULE: Self = Self(15000);
    /// Constant cost for a non-creation transaction
    pub const TX: Self = Self(21000);
    /// Constant cost for creation transaction
    pub const CREATION_TX: Self = Self(53000);
    /// Denominator of quadratic part of memory expansion gas cost
    pub const MEMORY_EXPANSION_QUAD_DENOMINATOR: Self = Self(512);
    /// Coefficient of linear part of memory expansion gas cost
    pub const MEMORY_EXPANSION_LINEAR_COEFF: Self = Self(3);
}

impl GasCost {
    /// Returns the `GasCost` as a `u64`.
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Returns the `GasCost` as a `usize`.
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<u8> for GasCost {
    fn from(cost: u8) -> Self {
        GasCost(cost as u64)
    }
}

impl From<u64> for GasCost {
    fn from(cost: u64) -> Self {
        GasCost(cost)
    }
}
