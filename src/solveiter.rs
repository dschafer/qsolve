use crate::{
    heuristic::{Heuristic, next_heuristic},
    solvestate::{SolveState, SolveStrategy},
};

/// This represents a stage in the process of solving a Queens board.
/// It contains both the current state of solving (solve_state), and
/// the next heuristic that will be used to advance the solving process
/// (next_heuristic). If no heuristic could be found (meaning either
/// the heuristic set is incomplete, or the board is solved), then
/// next_heuristic will be None.
pub struct SolveIterItem<'h, 'ss> {
    /// The current [SolveState] during the solving process.
    pub solve_state: SolveState<'ss>,

    /// The next [Heuristic] that we will apply to the given [SolveState],
    /// or None if no heuristic could be found.
    pub next_heuristic: Option<&'h dyn Heuristic>,
}

/// An Iterator that returns a series of StateIterItem's representing
/// the solving process for a given board.
pub struct SolveIter<'h, 'ss> {
    solve_state: SolveState<'ss>,
    solve_strategy: SolveStrategy,
    heuristics: &'h [Box<dyn Heuristic>],
    done: bool,
}
impl<'h, 'ss> Iterator for SolveIter<'h, 'ss> {
    type Item = SolveIterItem<'h, 'ss>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.solve_state.complete() {
            self.done = true;
            return Some(SolveIterItem {
                solve_state: self.solve_state.clone(),
                next_heuristic: None,
            });
        }
        let h = next_heuristic(&self.solve_state, self.solve_strategy, self.heuristics)?;
        let changes = h.changes(&self.solve_state)?;
        let old_solve_state = self.solve_state.clone();
        self.solve_state.apply_changes(&changes);
        Some(SolveIterItem {
            solve_state: old_solve_state,
            next_heuristic: Some(h),
        })
    }
}

/// Returns an Iterator that represents the solving of the provided
/// Queens solve state, using the given strategy and set of heuristics.
pub fn solve_iter<'h, 'b>(
    solve_state: SolveState<'b>,
    solve_strategy: SolveStrategy,
    heuristics: &'h [Box<dyn Heuristic>],
) -> SolveIter<'h, 'b> {
    SolveIter {
        solve_state,
        solve_strategy,
        heuristics,
        done: false,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;

    use crate::{board::Board, heuristic::all_heuristics};

    use super::*;

    #[test]
    fn solve_iter_succeeds() -> Result<()> {
        let board_str = "wwww\nwkkk\nrrrr\nbbbb";
        let board = Board::from_str(board_str)?;
        let solve_state = SolveState::from(&board);
        let heuristics = all_heuristics(&board);
        let mut solve_iter = solve_iter(solve_state, SolveStrategy::Fast, &heuristics);
        assert!(solve_iter.next().is_some());

        Ok(())
    }

    #[test]
    fn solve_iter_fails_on_impossible_board() -> Result<()> {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb"; // This board is not solvable, it has two solutions.
        let board = Board::from_str(board_str)?;
        let solve_state = SolveState::from(&board);
        let heuristics = all_heuristics(&board);
        let mut solve_iter = solve_iter(solve_state, SolveStrategy::Fast, &heuristics);
        assert!(solve_iter.next().is_none());

        Ok(())
    }
}
