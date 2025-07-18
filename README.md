# QSolve

_A command line tool and library for solving Queens puzzles_

[<img alt="crates.io" src="https://img.shields.io/crates/v/qsolve.svg?logo=rust" height="20">](https://crates.io/crates/qsolve)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-qsolve?logo=docs.rs" height="20">](https://docs.rs/qsolve)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dschafer/qsolve/rust.yml?branch=main&logo=github" height="20">](https://github.com/dschafer/qsolve/actions?query=branch%3Amain)
[<img alt="code coverage" src="https://img.shields.io/codecov/c/github/dschafer/qsolve?logo=codecov" height="20">](https://app.codecov.io/gh/dschafer/qsolve)

![An animation of a Queens puzzle being solved"](https://github.com/dschafer/qsolve/blob/main/media/li1-animate.gif?raw=true)

This command line tool and library is designed to solve [Queens puzzles](https://www.linkedin.com/games/queens),
with a few key characteristics:

- **Human-understandable**: Humans solve queens by iteratively eliminating and confirming squares. This library does the same process; it doesn't use and do any sort of search-algorithms to try and find the solution from afar.
- **Fast**: Within the bounds of the above, it tries to be as fast as possible. This means, for example, it uses bitfields rather than HashSets for efficient operations on small sets.
- **Tested**: While tracing code and error recovery means this library doesn't have 100% code coverage, it aspires to be as well-tested as possible. If `cargo test` passes, then we should be confident things work.
- **Documented**: The `qsolve` binary should have clear documentation available with `--help` for every subcommand. The `qsolve` library should have clear documentation (including doctests) for all public functionality.

## Installation

`qsolve` can be installed from [crates.io](https://crates.io/crates/qsolve) by running

```sh
cargo install qsolve
```

Alternately, the binary can be downloaded directly from [Github releases](https://github.com/dschafer/qsolve/releases).

## Command line example

Basic usage of the command line tool looks something like this:

```sh
qsolve solve games/linkedin-1-empty.txt --share
```

which yields the following screenshot (throughout this README, screenshots will be used since the command line tool relies heavily on ANSI color strings to output Queens boards):

![A terminal showing the results of running "qsolve solve games/linkedin-1-empty.txt --share"](https://github.com/dschafer/qsolve/blob/main/media/li1-solve-share.png?raw=true)

More compelling, though, is the `animate` subcommand, which doesn't just solve the puzzle, but walks you through the solution step by step.

https://github.com/user-attachments/assets/6b4d6798-63be-4000-b850-c8a45008dd1d

## Library Example

Basic usage of the library looks something like this:

```rust
use std::path::PathBuf;
use qsolve::heuristic::all_heuristics;
use qsolve::file::QueensFile;
use qsolve::solveiter::solve_iter;
use qsolve::solvestate::{SolveState, SolveStrategy};

fn solve() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a text file containing a Queens puzzle.
    let queens_file = QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;

    // Generate the initial solve state and print it.
    let solve_state = SolveState::from(&queens_file);
    println!("{}", solve_state);

    // Generate the list of heuristics to use to solve the puzzle.
    let heuristics = all_heuristics(solve_state.board);

    // Solve the puzzle and print out the solution.
    let solved = solve_iter(solve_state, SolveStrategy::Fast, &heuristics).last().unwrap().solve_state;
    println!("{}", solved);

    Ok(())
}
```

## Development

`qsolve` is a side project, so development will happen in a pretty ad-hoc basis (and issues and PRs might go unanswered: _caveat emptor_). However, if you wish to fork or contribute back, here's a quick runthrough:

This repository contains both [`qsolve` the binary](src/main.rs) and [`qsolve` the library it depends on](src/lib.rs). The only logic in the binary is command line logic; all actual functionality should live in the library.

There are moderately comprehensive integration, unit and doctests that can be run with `cargo test`. Additionally, there are a few benchmarks using the `criterion` benchmark engine that can be run with `cargo bench`. In general, changes should be neutral or positive
in that benchmark (for example, a change to use the `bitvec` package to implement the data structures in [`src/datastructure.rs`](src/datastructure.rs) was abandoned because `cargo bench` showed it was a regression).
