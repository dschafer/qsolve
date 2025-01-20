use std::{ffi::OsStr, fs};

use anyhow::Result;

use qsolve::{file::QueensFile, solvestate::SolveState};

#[test]
fn reads_all_images() -> Result<()> {
    for dir_entry in fs::read_dir("games/")? {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        let extension = path.extension().and_then(OsStr::to_str);
        if extension != Some("png") && extension != Some("jpg") && extension != Some("webp") {
            continue;
        }
        let queens_file = QueensFile::try_from_image_file(&path)?;
        let solve_state = SolveState::from(&queens_file);
        assert!(
            solve_state.is_valid(),
            "Testing initial state validity for {:?}",
            dir_entry.path()
        );
    }
    Ok(())
}
