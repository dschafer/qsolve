#![warn(missing_docs)]

//! A library for solving Queens puzzles
//!
//! This library is designed to solve [Queens puzzles](<https://www.linkedin.com/games/queens>),
//! with a few key characteristics:
//!
//! * **Human-understandable**: Humans solve queens by iteratively eliminating and confirming squares. This library does the same process; it doesn't use and do any sort of search-algorithms to try and find the solution from afar.
//! * **Speed**: Within the bounds of the above, it tries to be as fast as possible. This means, for example, it uses bitfields rather than HashSets for efficient operations on small sets.
//! - **Tested**: While tracing code and error recovery means this library doesn't have 100% code coverage, it aspires to be as well-tested as possible. If `cargo test` passes, then we should be confident things work.
//! - **Documented**: The `qsolve` binary should have clear documentation available with `--help` for every subcommand. The `qsolve` library should have clear documentation (including doctests) for all public functionality.
//!
//! # Example
//!
//! Basic usage of the library looks something like this:
//!
//! ```
//! # use std::path::PathBuf;
//! # use qsolve::file::QueensFile;
//! # use qsolve::heuristic::all_heuristics;
//! # use qsolve::solveiter::solve_iter;
//! # use qsolve::solvestate::{SolveState, SolveStrategy};
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//! // Parse a text file containing a Queens puzzle.
//! let queens_file = QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;
//!
//! // Generate the initial solve state and print it.
//! let solve_state = SolveState::from(&queens_file);
//! println!("{}", solve_state);
//!
//! // Generate the list of heuristics to use to solve the puzzle.
//! let heuristics = all_heuristics(solve_state.board);
//!
//! // Solve the puzzle and print out the solution.
//! let solved = solve_iter(solve_state, SolveStrategy::Fast, &heuristics).last().unwrap().solve_state;
//! println!("{}", solved);
//! # Ok(())
//! # }
//! ```

/// Structs to represent Queens boards.
pub mod board;

/// Data structures for efficient manipuations of rows, cols, colors and coords.
pub mod datastructure;

/// Logic to represent an underlying file containing a Queens game.
pub mod file;

/// Heuristics used to solve the Queens game.
pub mod heuristic;

/// Image parsing logic to allow screenshots of Queens games to be used.
pub mod image;

/// Iterators for moving through the process of solving a game.
pub mod solveiter;

/// Structs to represent intermediate states of solving a Queens puzzle.
pub mod solvestate;

/// Representation of different square colors and associated display logic.
pub mod squarecolor;

/// Logic to generate the share text for a solved puzzle.
pub mod share;

// Use doc_comment to ensure code snippets in the readme compile.
extern crate doc_comment;
doc_comment::doctest!("../README.md");
