use std::time::Duration;

use crate::{heuristic::Changes, solveiter::SolveIterItem};

/// Generates the share text for a solved puzzle.
///
/// # Arguments
/// * `state_iter_items` - A slice of [SolveIterItem]s that show that path to solve the puzzle.
/// * `puzzle_name` - The name of the puzzle to include in the share text.
/// * `elapsed` - A [Duration] that represents how long the puzzle took to solve.
///
/// # Returns
/// A three-line string that serves as the "share text" for the solved puzzle.
///
/// # Examples
/// ```
/// # use std::path::PathBuf;
/// # use std::time::Instant;
/// # use qsolve::heuristic::all_heuristics;
/// # use qsolve::file::QueensFile;
/// # use qsolve::share::generate_share_content;
/// # use qsolve::solveiter::solve_iter;
/// # use qsolve::solvestate::{SolveState, SolveStrategy};
/// # fn solve() -> Result<(), Box<dyn std::error::Error>> {
///     // Solve a puzzle, while timing it.
///     let start_time = Instant::now();
///     let queens_file = QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;
///     let solve_state = SolveState::from(&queens_file);
///     let heuristics = all_heuristics(solve_state.board);
///     let solve_vec = solve_iter(solve_state, SolveStrategy::Fast, &heuristics).collect::<Vec<_>>();
///     let elapsed = start_time.elapsed();
///
///     // Generate and print the share content.
///     let share_content = generate_share_content(&solve_vec, "Linked In #1", elapsed);
///     println!("{}", share_content);
/// #   Ok(())
/// # }
/// ```
pub fn generate_share_content(
    state_iter_items: &[SolveIterItem],
    puzzle_name: &str,
    elapsed: Duration,
) -> String {
    let queens_order = state_iter_items
        .iter()
        .filter_map(
            |SolveIterItem {
                 solve_state,
                 next_heuristic,
             }| {
                let h = (*next_heuristic)?;
                if let Some(Changes::AddQueen { queen, x: _ }) = h.changes(solve_state) {
                    return Some(solve_state.board.color(&queen));
                }
                None
            },
        )
        .collect::<Vec<_>>();

    let puzzle_name = if puzzle_name.chars().all(char::is_numeric) {
        format!("#{puzzle_name}")
    } else {
        puzzle_name.to_string()
    };

    let mut output = String::new();
    output.push_str(&format!(
        "QSolve {puzzle_name} | {elapsed:?} and flawless\n"
    ));
    output.push_str(&format!(
        "First \u{1f451}s: {}\n",
        &queens_order[0..3]
            .iter()
            .map(|x| (*x).to_unicode_block().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    ));
    output.push_str("github.com/dschafer/qsolve");

    output
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::Result;

    use crate::{
        file::QueensFile,
        heuristic::all_heuristics,
        solveiter::solve_iter,
        solvestate::{SolveState, SolveStrategy},
    };

    use super::*;

    #[test]
    fn generate_share_content_works() -> Result<()> {
        let queens_file =
            QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;
        let solve_state = SolveState::from(&queens_file);
        let heuristics = all_heuristics(solve_state.board);
        let state_iter_items =
            solve_iter(solve_state, SolveStrategy::Fast, &heuristics).collect::<Vec<_>>();

        let share_text =
            generate_share_content(&state_iter_items, "LinkedIn #1", Duration::from_secs(1));
        let share_lines = share_text.lines().collect::<Vec<_>>();
        assert_eq!(share_lines.len(), 3);
        assert_eq!(share_lines[0], "QSolve LinkedIn #1 | 1s and flawless");
        assert_eq!(
            share_lines[1],
            "First \u{1f451}s: \u{1F7E8} \u{2B1C} \u{1F7EA}"
        );
        assert_eq!(share_lines[2], "github.com/dschafer/qsolve");

        Ok(())
    }

    #[test]
    fn generate_share_content_puzzle_number() -> Result<()> {
        let queens_file =
            QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;
        let solve_state = SolveState::from(&queens_file);
        let heuristics = all_heuristics(solve_state.board);
        let state_iter_items =
            solve_iter(solve_state, SolveStrategy::Fast, &heuristics).collect::<Vec<_>>();

        let share_text = generate_share_content(&state_iter_items, "1234", Duration::from_secs(1));
        let share_lines = share_text.lines().collect::<Vec<_>>();
        assert_eq!(share_lines.len(), 3);
        assert_eq!(share_lines[0], "QSolve #1234 | 1s and flawless");

        Ok(())
    }
}
