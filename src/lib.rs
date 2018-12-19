//!
//! Provides an algorithm which matches execution blocks inside a program with
//! some registry of known execution blocks.
//!

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::BuildHasher;

/// Finds matches of execution blocks inside a program with the specified
/// registry of known blocks. For each block in the program returns a vector with
/// block start position wrapped by 'Matched' if a block was found in the registry,
/// 'NotMatched' if a block wasn't found in the registry. Note that order in
/// the result vector is corresponded to order of 'End' instructions for each block.
/// It means that at the first position of the result vector will be placed the
/// first closed block (not the first started block).
///
/// # Arguments
///
/// * known_blocks - The registry of known execution blocks.
/// * program - The program is a vector of blocks for matching with the registry.
///
pub fn find_matches<S: BuildHasher>(
    known_blocks: &HashSet<&[Instruction], S>,
    program: &[Instruction],
) -> Result<Vec<BlockInfo>, MatchError> {
    use self::Instruction::*;

    if program.is_empty() {
        return Err(MatchError::NoOneBlockFound);
    }

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

                        let result = if known_blocks.contains(block) {
                            BlockInfo::Matched(block_start_idx)
                        } else {
                            BlockInfo::NotMatched(block_start_idx)
                        };

                        Ok(result)
                    })
                    .or_else(|| Some(Err(MatchError::InvalidBlock("".to_string())))),
                _ => None, // do nothing
            }
        })
        .collect();

    if block_stack.is_empty() {
        result
    } else {
        Err(MatchError::InvalidBlock("".to_string()))
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

/// Indicates that a block was/wasn't matched with some block from known blocks.
/// Also contains an index of first instruction for this block in the whole program.
#[derive(Debug, PartialOrd, PartialEq)]
pub enum BlockInfo {
    Matched(usize),
    NotMatched(usize),
}

impl BlockInfo {
    #[allow(dead_code)]
    pub fn start_position(&self) -> Result<usize, MatchError> {
        match self {
            BlockInfo::Matched(pos) | BlockInfo::NotMatched(pos) => Ok(pos.to_owned()),
        }
    }
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
    use crate::BlockInfo::*;
    use crate::Instruction;
    use crate::Instruction::*;
    use crate::MatchError;
    use std::collections::HashSet;

    fn default_register() -> HashSet<&'static [Instruction]> {
        let mut known_blocks: HashSet<&[Instruction]> = HashSet::new();
        known_blocks.insert(&[Begin, Push(1), End]);
        known_blocks.insert(&[If, Push(2), Not, Push(3), End]);
        known_blocks.insert(&[If, Push(2), Push(3), End]);
        known_blocks.insert(&[If, End]);
        known_blocks
    }

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
        let program = vec![Or, End];

        let result = find_matches(&register, &program);

        assert_eq!("Invalid block: ", result.unwrap_err().to_string())
    }

    #[test]
    fn not_closed_block() {
        let register = default_register();
        let program = vec![Begin];

        let result = find_matches(&register, &program);

        assert_eq!("Invalid block: ", result.unwrap_err().to_string())
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

        let expected = vec![Matched(1), Matched(6), NotMatched(5), NotMatched(0)];
        let result = find_matches(&known_blocks, &program);

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn big_correct_program() {
        let deep_lvl = 1000;
        let register = default_register();
        let program = create_program(deep_lvl);

        let result = find_matches(&register, &program).unwrap();

        assert_eq!(deep_lvl, result.len() - 1);
        assert_eq!(&NotMatched(0), result.get(deep_lvl).unwrap());
        assert_eq!(&Matched(deep_lvl * 2 - 1), result.get(0).unwrap());
    }

    fn create_program(deep_lvl: usize) -> Vec<Instruction> {
        let mut program = vec![Begin];
        for _idx in 0..deep_lvl {
            program.push(If);
            program.push(Push(2));
        }
        for _idx in 0..deep_lvl {
            program.push(Push(3));
            program.push(End);
        }
        program.push(End);
        program
    }

}
