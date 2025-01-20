use std::path::PathBuf;

use anyhow::Result;

use qsolve::{
    file::QueensFile,
    heuristic::all_heuristics,
    solveiter::solve_iter,
    solvestate::{SolveState, SolveStrategy},
};

#[test]
fn all_strategies_solve() -> Result<()> {
    let queens_file = QueensFile::try_from_text_file(&PathBuf::from("games/linkedin-1-empty.txt"))?;
    for strategy in [
        SolveStrategy::Fast,
        SolveStrategy::Short,
        SolveStrategy::Simple,
    ] {
        let solve_state = SolveState::from(&queens_file);
        let heuristics = all_heuristics(solve_state.board);
        let state_iter_items = solve_iter(solve_state, strategy, &heuristics).collect::<Vec<_>>();
        let final_state = &state_iter_items.iter().last().unwrap().solve_state;
        assert!(final_state.complete());
    }
    Ok(())
}
