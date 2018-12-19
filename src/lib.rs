//!
//! Provides an algorithm which matches execution blocks inside a program with
//! some registry of known execution blocks.
//!

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

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
pub fn find_matches(
    known_blocks: &HashSet<&[Instruction]>,
    program: &[Instruction],
) -> Result<Vec<BlockInfo>, MatchError> {
    use self::Instruction::*;

    // todo try to handle in generic maner
    //    if program.get(0) != Some(&Begin) {
    //        return Err(MatchError);  // todo
    //    }

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
                End => {
                    block_stack.pop().map(|block_start_idx| {
                        let block = &program[block_start_idx..=ins_idx];

                        if known_blocks.contains(block) {
                            BlockInfo::Matched(block_start_idx)
                        } else {
                            BlockInfo::NotMatched(block_start_idx)
                        }
                    })

                    // todo handle invalid format
                }
                _ => None, // do nothing
            }
        })
        .collect();

    Ok(result)
}

// todo functional version

// todo ?
#[derive(Debug)]
pub struct MatchError;

// case 1 Missing Begin Instrustion
// case 2 Missing End instruction
// better
// Number of Instructions that open block isn't correspond number of

impl Error for MatchError {}

impl Display for MatchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Match error!")
    }
}

/// Vm Instruction set.
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

/// Contains information about matching Block.
// todo
#[derive(Debug, PartialOrd, PartialEq)]
pub enum BlockInfo {
    /// Tells that block matched with
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

// todo write tests
#[cfg(test)]
mod tests {
    use crate::find_matches;
    use crate::BlockInfo::*;
    use crate::Instruction;
    use crate::Instruction::*;
    use std::collections::HashSet;

    #[test]
    fn no_blocks_found() {
        // should returns Error
    }

    #[test]
    fn wrong_first_ins() {
        // should be Begin only
    }

    #[test]
    fn not_closed_block() {
        // all blocks should be closed/valid
    }

    #[test]
    fn correct_program() {
        let mut known_blocks: HashSet<&[Instruction]> = HashSet::new();
        known_blocks.insert(&[Begin, Push(1), End]);
        known_blocks.insert(&[If, Push(2), Not, Push(3), End]);
        known_blocks.insert(&[If, Push(2), Push(3), End]);
        known_blocks.insert(&[If, End]);
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
        // generates a big program with deep level of recursion

    }
}
