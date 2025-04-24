use std::{
    ffi::OsStr,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossterm::{
    cursor::{Hide, MoveUp, Show},
    execute,
    style::Print,
    terminal::Clear,
};
use log::debug;
use qsolve::heuristic::{Heuristic, all_heuristics};
use qsolve::share::generate_share_content;
use qsolve::solvestate::{Charset, SolveState, SolveStrategy};
use qsolve::{datastructure::CoordSet, solveiter::SolveIterItem};
use qsolve::{file::QueensFile, solveiter::solve_iter};

#[derive(Parser)]
#[command(version, about, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Prints out the board
    Print {
        #[command(flatten)]
        path_args: PathCli,

        #[command(flatten)]
        display_args: DisplayCli,
    },

    /// Display an animation of the solving of the board
    Animate {
        #[command(flatten)]
        path_args: PathCli,

        #[command(flatten)]
        display_args: DisplayCli,

        #[command(flatten)]
        solve_args: SolveCli,

        /// The length of delay between animation steps, in ms
        #[clap(long, value_parser = |s: &str| s.parse().map(Duration::from_millis), default_value = "500")]
        delay: Duration,
    },

    /// Solve the board and display the solution
    Solve {
        #[command(flatten)]
        path_args: PathCli,

        #[command(flatten)]
        display_args: DisplayCli,

        #[command(flatten)]
        solve_args: SolveCli,

        /// Generate a share text, with the provided string as the name
        #[clap(long, num_args = 0..=1, require_equals = true, default_missing_value = "")]
        share: Option<String>,
    },

    /// Solve boards repeatedly for profiling
    Profile {
        #[command(flatten)]
        path_args: PathCli,

        #[command(flatten)]
        solve_args: SolveCli,

        /// How many iterations to run
        #[clap(long, default_value_t = 1)]
        iterations: usize,
    },

    /// Provide a hint about the next move on the board
    Hint {
        #[command(flatten)]
        path_args: PathCli,

        #[command(flatten)]
        display_args: DisplayCli,

        #[command(flatten)]
        solve_args: SolveCli,

        /// The type of hint that should be provided
        #[clap(long, default_value = "both")]
        hint_type: HintType,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum HintType {
    /// Show only the heuristic used, and not the resulting change.
    Heuristic,
    /// Show only the resulting change, and not the heuristic used.
    Result,
    /// Show both the heuristic used and the resulting change.
    #[default]
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
/// What type of file to read
enum FileType {
    /// Automatically detect based on file extension
    #[default]
    Auto,
    /// Force text file parsing
    Text,
    /// Force image file parsing
    Image,
}

#[derive(Args, Debug)]
struct PathCli {
    /// The path to the file containing the board
    path: std::path::PathBuf,

    /// What type of file to read
    #[clap(long, default_value = "auto")]
    file_type: FileType,

    /// Whether we should use the Queens and Xs in the file, or
    /// clear it to be an empty board
    #[clap(long, default_value = "false")]
    clear: bool,
}

#[derive(Args, Debug)]
struct DisplayCli {
    #[clap(long, default_value = "unicode")]
    /// What charset to use when displaying the board
    charset: Charset,
}

#[derive(Args, Debug)]
struct SolveCli {
    #[clap(long, default_value = "fast")]
    /// What strategy to use for solving the puzzle
    strategy: SolveStrategy,
}

fn queens_file_from_path(path_args: &PathCli) -> Result<QueensFile> {
    let qf = match path_args.file_type {
        FileType::Text => QueensFile::try_from_text_file(&path_args.path),
        FileType::Image => QueensFile::try_from_image_file(&path_args.path),
        FileType::Auto => QueensFile::try_from_text_file(&path_args.path)
            .or_else(|_| QueensFile::try_from_image_file(&path_args.path)),
    }?;
    if path_args.clear {
        Ok(QueensFile {
            board: qf.board,
            squares: None,
        })
    } else {
        Ok(qf)
    }
}

/// Top-level entry point for the print subcommand.
fn print(path_args: &PathCli, display_args: &DisplayCli) -> Result<()> {
    let queens_file = queens_file_from_path(path_args)?;
    let solve_state = SolveState::from(&queens_file);
    println!(
        "{}",
        solve_state.ansi_string(CoordSet::default(), display_args.charset)?
    );
    Ok(())
}

/// Helper function to print a given [SolveIterItem] as part of the
/// animate command.
fn print_animated_iter_item(
    solve_iter_item: &SolveIterItem,
    charset: Charset,
    delay: Duration,
) -> Result<()> {
    let mut stdout = std::io::stdout();
    let size: u16 = (solve_iter_item.solve_state.board.size())
        .try_into()
        .unwrap();

    execute!(
        stdout,
        Print(
            solve_iter_item
                .solve_state
                .ansi_string(CoordSet::default(), charset)
                .unwrap()
        ),
        Print("\n"),
    )?;
    std::thread::sleep(delay);
    execute!(
        stdout,
        MoveUp(size),
        Print(
            solve_iter_item
                .solve_state
                .ansi_string(
                    solve_iter_item
                        .next_heuristic
                        .map(|h| h.seen_coords(&solve_iter_item.solve_state))
                        .unwrap_or_default(),
                    charset
                )
                .unwrap()
        ),
        Print("\n"),
        Clear(crossterm::terminal::ClearType::CurrentLine),
        Print(
            solve_iter_item
                .next_heuristic
                .map_or("Done!\n".to_string(), Heuristic::description)
        ),
        Print("\n"),
    )?;
    if solve_iter_item.next_heuristic.is_none() {
        return Ok(());
    }
    std::thread::sleep(delay);
    execute!(
        stdout,
        MoveUp(1),
        Clear(crossterm::terminal::ClearType::CurrentLine),
        MoveUp(1),
        Clear(crossterm::terminal::ClearType::CurrentLine),
        MoveUp(size),
    )?;
    Ok(())
}

/// Top-level entry point for the animate subcommand.
fn animate(
    path_args: &PathCli,
    display_args: &DisplayCli,
    solve_args: &SolveCli,
    delay: &Duration,
) -> Result<()> {
    let queens_file = queens_file_from_path(path_args)?;
    let solve_state = SolveState::from(&queens_file);
    let heuristics = all_heuristics(solve_state.board);

    let mut stdout = std::io::stdout();
    execute!(stdout, Hide)?;

    for solve_iter_item in solve_iter(solve_state, solve_args.strategy, &heuristics) {
        print_animated_iter_item(&solve_iter_item, display_args.charset, *delay)?;
    }
    execute!(stdout, Show)?;
    Ok(())
}

/// Top-level entry point for the solve subcommand.
fn solve(
    path_args: &PathCli,
    display_args: &DisplayCli,
    solve_args: &SolveCli,
    share: &Option<String>,
) -> Result<()> {
    let start_time = Instant::now();
    let queens_file = queens_file_from_path(path_args)?;
    let solve_state = SolveState::from(&queens_file);
    let heuristics = all_heuristics(solve_state.board);
    let state_iter_items =
        solve_iter(solve_state, solve_args.strategy, &heuristics).collect::<Vec<_>>();
    let final_state = &state_iter_items.iter().last().unwrap().solve_state;
    let elapsed = start_time.elapsed();
    println!(
        "{}",
        final_state.ansi_string(CoordSet::default(), display_args.charset)?
    );
    debug!("Solve complete.");
    if let Some(share_text) = share {
        debug!("Generating share text.");
        let puzzle_name = if !share_text.is_empty() {
            share_text.clone()
        } else {
            path_args
                .path
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_string()
        };
        println!(
            "{}",
            generate_share_content(&state_iter_items, &puzzle_name, elapsed)
        );
    }
    Ok(())
}

/// Top-level entry point for the hint subcommand.
fn hint(
    path_args: &PathCli,
    display_args: &DisplayCli,
    solve_args: &SolveCli,
    hint_type: &HintType,
) -> Result<()> {
    let queens_file = queens_file_from_path(path_args)?;
    let solve_state = SolveState::from(&queens_file);
    let heuristics = all_heuristics(solve_state.board);
    let mut state_iter_items = solve_iter(solve_state, solve_args.strategy, &heuristics);
    let next_item = state_iter_items.next();
    let Some(next_item) = next_item else {
        println!("No next step found.");
        return Ok(());
    };
    let Some(next_heuristic) = next_item.next_heuristic else {
        println!("No next step found.");
        return Ok(());
    };
    if hint_type == &HintType::Both || hint_type == &HintType::Heuristic {
        println!(
            "{}",
            next_item
                .solve_state
                .ansi_string(
                    next_heuristic.seen_coords(&next_item.solve_state),
                    display_args.charset
                )
                .unwrap()
        );
        println!("{}", next_heuristic.description());
    }
    if hint_type == &HintType::Both || hint_type == &HintType::Result {
        let changes = next_heuristic.changes(&next_item.solve_state);
        let Some(changes) = changes else {
            println!("No next step found.");
            return Ok(());
        };
        let following_item = state_iter_items.next();
        let Some(following_item) = following_item else {
            println!("No next step found.");
            return Ok(());
        };
        println!(
            "{}",
            following_item
                .solve_state
                .ansi_string(changes.changed_coords(), display_args.charset)
                .unwrap()
        );
    }
    Ok(())
}

/// Top-level entry point for the profile subcommand.
fn profile(path_args: &PathCli, solve_args: &SolveCli, iterations: &usize) -> Result<()> {
    let start_time = Instant::now();
    for _ in 0..*iterations {
        let queens_file = queens_file_from_path(path_args)?;
        let solve_state = SolveState::from(&queens_file);
        let heuristics = all_heuristics(solve_state.board);
        solve_iter(solve_state, solve_args.strategy, &heuristics).for_each(drop);
    }
    let elapsed = start_time.elapsed();
    println!("{} iterations completed in {:?}", iterations, elapsed);
    Ok(())
}

/// Top-level entry point for the program.
fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    debug!("Running command {:?}", cli.command);
    match &cli.command {
        Commands::Print {
            path_args,
            display_args,
        } => print(path_args, display_args),
        Commands::Animate {
            path_args,
            display_args,
            solve_args,
            delay,
        } => animate(path_args, display_args, solve_args, delay),
        Commands::Solve {
            path_args,
            display_args,
            solve_args,
            share,
        } => solve(path_args, display_args, solve_args, share),
        Commands::Profile {
            path_args,
            solve_args,
            iterations,
        } => profile(path_args, solve_args, iterations),
        Commands::Hint {
            path_args,
            display_args,
            solve_args,
            hint_type,
        } => hint(path_args, display_args, solve_args, hint_type),
    }?;

    Ok(())
}
