use std::{ffi::OsStr, fs};

use anyhow::Result;

use qsolve::{
    file::QueensFile,
    heuristic::all_heuristics,
    solveiter::solve_iter,
    solvestate::{SolveState, SolveStrategy},
};

#[test]
fn solves_all_folder() -> Result<()> {
    for dir_entry in fs::read_dir("games/")? {
        let dir_entry = dir_entry?;
        if dir_entry.path().extension().and_then(OsStr::to_str) != Some("txt") {
            continue;
        }
        let queens_file = QueensFile::try_from_text_file(&dir_entry.path())?;
        let solve_state = SolveState::from(&queens_file);
        let heuristics = all_heuristics(solve_state.board);
        let state_iter_items =
            solve_iter(solve_state, SolveStrategy::Fast, &heuristics).collect::<Vec<_>>();
        let final_state = &state_iter_items.iter().last().unwrap().solve_state;
        assert!(
            final_state.complete(),
            "Testing final state completion for {:?}",
            dir_entry.path()
        );
    }
    Ok(())
}
