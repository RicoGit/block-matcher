//!
//! Provides an algorithm which matches execution blocks inside a program with
//! some registry of known execution blocks.
//!

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Finds matches of execution blocks inside a program with the specified
/// registry of known blocks. For each block in the program returns a vector with
/// block start position and block index in the registry. Note that order in
/// the result vector is corresponded to order of 'End' instructions for each block.
/// It means that at the first position of the result vector will be placed the
/// first closed block (not the first started block).
///
/// # Arguments
///
/// * known_blocks - The registry of known execution blocks.
/// * program - The program is a vector of blocks for matching with the registry.
///
pub fn find_matches(
    known_blocks: &[&[Instruction]],
    program: &[Instruction],
) -> Result<Vec<BlockInfo>, MatchError> {
    use self::Instruction::*;

    if program.is_empty() {
        return Err(MatchError::NoOneBlockFound);
    }

    let register: HashMap<&[Instruction], usize> = known_blocks
        .iter()
        .enumerate()
        .map(|(idx, ins)| (*ins, idx))
        .collect();

    let mut block_stack = Vec::new();

    let result = program
        .iter()
        .enumerate()
        .filter_map(|(ins_idx, instruction)| {
            match instruction {
                Begin | If => {
                    block_stack.push(ins_idx);
                    None
                }
                End => block_stack
                    .pop()
                    .map(|block_start_idx| {
                        let block = &program[block_start_idx..=ins_idx];

                        Ok(BlockInfo {
                            block_start_idx,
                            registry_idx: register.get(block).map(Clone::clone),
                        })
                    })
                    .or_else(|| {
                        let msg = format!(
                            "Attempt to close the non-existent block, at the position: {}",
                            ins_idx
                        );
                        Some(Err(MatchError::InvalidBlock(msg)))
                    }),
                _ => None, // do nothing
            }
        })
        .collect();

    if block_stack.is_empty() {
        result
    } else {
        let msg = format!(
            "Next blocks weren't be closed. The start blocks positions: {:?}",
            block_stack
        );
        Err(MatchError::InvalidBlock(msg))
    }
}

/// VM instruction set.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Push(usize),
    Or,
    And,
    Not,
    If,
    Begin,
    End,
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct BlockInfo {
    /// An index of first block instruction in the whole program
    block_start_idx: usize,
    /// An index of this block in registry
    registry_idx: Option<usize>,
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum MatchError {
    NoOneBlockFound,
    InvalidBlock(String),
}

impl Error for MatchError {}

impl Display for MatchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MatchError::NoOneBlockFound => {
                write!(f, "Input program should contain at least one block")
            }
            MatchError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::find_matches;
    use crate::BlockInfo;
    use crate::Instruction;
    use crate::Instruction::*;
    use crate::MatchError;

    #[test]
    fn no_blocks_found() {
        let register = default_register();
        let program = vec![];

        let result = find_matches(&register, &program);

        assert_eq!(MatchError::NoOneBlockFound, result.unwrap_err())
    }

    #[test]
    fn not_opened_block() {
        let register = default_register();
        let program = vec![Or, End, Push(1), End];

        let result = find_matches(&register, &program);

        assert_eq!(
            "Invalid block: Attempt to close the non-existent block, at the position: 1",
            result.unwrap_err().to_string()
        )
    }

    #[test]
    fn not_closed_block() {
        let register = default_register();
        let program = vec![Begin, Push(2), If, End, If];

        let result = find_matches(&register, &program);

        assert_eq!(
            "Invalid block: Next blocks weren't be closed. The start blocks positions: [0, 4]",
            result.unwrap_err().to_string()
        )
    }

    #[test]
    fn correct_program() {
        let known_blocks = default_register();

        let program = vec![
            Begin,
            If,
            Push(2),
            Push(3),
            End,
            If,
            If,
            End,
            Push(1),
            End,
            End,
        ];

        let expected = vec![matched(1, 2), matched(6, 3), not_matched(5), not_matched(0)];
        let result = find_matches(&known_blocks, &program);

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn big_correct_program() {
        let deeps_lvl = 1000;
        let register = default_register();
        let program = create_program(deeps_lvl);

        let result = find_matches(&register, &program).unwrap();

        assert_eq!(deeps_lvl, result.len() - 1);
        assert_eq!(&not_matched(0), result.get(deeps_lvl).unwrap());
        assert_eq!(&matched(deeps_lvl * 2 - 1, 2), result.get(0).unwrap());
    }

    fn matched(block_start_idx: usize, registry_idx: usize) -> BlockInfo {
        BlockInfo {
            block_start_idx,
            registry_idx: Some(registry_idx),
        }
    }

    fn not_matched(block_start_idx: usize) -> BlockInfo {
        BlockInfo {
            block_start_idx,
            registry_idx: None,
        }
    }

    fn default_register() -> Vec<&'static [Instruction]> {
        let mut known_blocks: Vec<&'static [Instruction]> = Vec::new();
        known_blocks.push(&[Begin, Push(1), End]);
        known_blocks.push(&[If, Push(2), Not, Push(3), End]);
        known_blocks.push(&[If, Push(2), Push(3), End]);
        known_blocks.push(&[If, End]);
        known_blocks
    }

    /// Creates a program with specified deeps level.
    fn create_program(deeps_lvl: usize) -> Vec<Instruction> {
        let mut program = vec![Begin];
        for _idx in 0..deeps_lvl {
            program.push(If);
            program.push(Push(2));
        }
        for _idx in 0..deeps_lvl {
            program.push(Push(3));
            program.push(End);
        }
        program.push(End);
        program
    }
}
