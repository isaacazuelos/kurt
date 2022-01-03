//! The instructions our VM will use.

use crate::constant::Index;

/// These are the individual instructions that our VM interprets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    True,
    False,
    Unit,

    LoadConstant(Index),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_size() {
        // We want our instructions to be 64-bits.
        assert!(std::mem::size_of::<Op>() == std::mem::size_of::<u64>());
        // We want our instructions to be word-sized.
        assert!(std::mem::size_of::<Op>() <= std::mem::size_of::<usize>());
        // Yes, this means we expect to be on a 64-bit machine.
    }
}
