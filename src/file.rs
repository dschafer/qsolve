use std::{fmt::Display, str::FromStr};

use anyhow::{Context, Result, bail, ensure};
use image::ImageReader;

use crate::{
    board::Board,
    image::analyze_grid_image,
    solvestate::{Charset, SquareVal},
};

/// This represents a solve state as part of an input file.
#[derive(Clone, Debug)]
pub struct InputSquares(pub Vec<Option<SquareVal>>);

impl Display for InputSquares {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = (self.0.len() as f64).sqrt() as usize;
        for r in 0..size {
            for c in 0..size {
                write!(
                    f,
                    "{}",
                    SquareVal::as_char(self.0[size * r + c], false, &Charset::Ascii)
                )?
            }
            if r != size - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl From<Vec<Option<SquareVal>>> for InputSquares {
    fn from(value: Vec<Option<SquareVal>>) -> Self {
        InputSquares(value)
    }
}

impl From<&Board> for InputSquares {
    fn from(board: &Board) -> Self {
        InputSquares(vec![None; board.square_count()])
    }
}

impl From<InputSquares> for Vec<Option<SquareVal>> {
    fn from(val: InputSquares) -> Self {
        val.0
    }
}

impl FromStr for InputSquares {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let lines = s.trim().lines().collect::<Vec<_>>();
        let size = lines.len();
        for (line_num, &line) in lines.iter().enumerate() {
            if line.len() != size {
                let row_num = line_num + 1;
                let row_len = line.len();
                bail!(
                    "Invalid solve state squares: row {row_num} has {row_len} entries but the board is {size} rows long."
                );
            }
        }
        let solve_state_squares = lines
            .into_iter()
            .flat_map(|s| s.chars())
            .map(SquareVal::try_from)
            .collect::<Result<_>>()?;
        Ok(InputSquares(solve_state_squares))
    }
}

/// This represents a parsed input file.
#[derive(Debug)]
pub struct QueensFile {
    /// The [Board] parsed from the input file.
    pub board: Board,

    /// An optional partial solution from the input file.
    ///
    /// Some inputs only contain a board, in which case this will be None.
    /// But if the input file has a partial solution as well, this will
    /// instead be Some(s) where s is that partial solution.
    pub squares: Option<InputSquares>,
}

impl QueensFile {
    /// This reads the given path as a text file and attempts to return
    /// a QueensFile from it.
    pub fn try_from_text_file(path: &std::path::PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Could not read file `{path:?}`"))?;

        QueensFile::from_str(&content)
            .with_context(|| format!("Failed to create board from text file at {path:?}"))
    }

    /// This reads the given path as an image file and attempts to return
    /// a QueensFile from it.
    pub fn try_from_image_file(path: &std::path::PathBuf) -> Result<Self> {
        let rgb_image = ImageReader::open(path)?.decode()?.to_rgb8();

        analyze_grid_image(&rgb_image)
            .with_context(|| format!("Failed to create board from image at {path:?}"))
    }
}

impl FromStr for QueensFile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let lines = s.trim().lines().collect::<Vec<_>>();
        let lines_len = lines.len();
        ensure!(lines_len != 0, "Invalid solve state: no lines found.");

        let size = lines[0].len();
        let is_squares_formatted = lines_len == (1 + size * 2) && lines[size].is_empty();
        ensure!(
            lines_len == size || is_squares_formatted,
            "Invalid solve state: {lines_len} lines for size {size}."
        );

        let board = Board::from_str(&lines[0..size].join("\n"))?;
        let squares = if is_squares_formatted {
            Some(InputSquares::from_str(
                &lines[(size + 1)..(size * 2 + 1)].join("\n"),
            )?)
        } else {
            None
        };

        Ok(QueensFile { board, squares })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_squares_from_str() -> Result<()> {
        let input_str = "Qxxx\nxx..\nx...\nx...";
        let input_squares = InputSquares::from_str(input_str)?;
        assert_eq!(input_squares.0.len(), 16);
        Ok(())
    }

    #[test]
    fn input_squares_display() -> Result<()> {
        let input_str = "Qxxx\nxx..\nx...\nx...";
        let input_squares = InputSquares::from_str(input_str)?;
        assert_eq!(format!("{input_squares}"), input_str.replace(".", " "));
        Ok(())
    }

    #[test]
    fn input_squares_into() -> Result<()> {
        let input_str = "Qxxx\nxx..\nx...\nx...";
        let input_squares = InputSquares::from_str(input_str)?;
        let square_vec: Vec<Option<SquareVal>> = input_squares.into();
        assert_eq!(square_vec.len(), 16);
        Ok(())
    }

    #[test]
    fn input_squares_from_invalid_str() {
        let input_str = "Qxxxxxxxxxx\nxx..\nx...\nx...";
        let input_result = InputSquares::from_str(input_str);
        assert!(input_result.is_err());
    }

    #[test]
    fn input_squares_from_invalid_char_str() {
        let input_str = "ZZZZ\nxx..\nx...\nx...";
        let input_result = InputSquares::from_str(input_str);
        assert!(input_result.is_err());
    }

    #[test]
    fn input_squares_from_board_str() -> Result<()> {
        let file_str = "wwww\nkkkk\nrrrr\nbbbb";
        let file = QueensFile::from_str(file_str)?;
        let input_squares = InputSquares::from(&file.board);
        assert_eq!(input_squares.0.len(), 16);
        Ok(())
    }

    #[test]
    fn queens_file_from_board_str() -> Result<()> {
        let file_str = "wwww\nkkkk\nrrrr\nbbbb";
        let file = QueensFile::from_str(file_str)?;
        assert_eq!(file.board.size(), 4);
        assert!(file.squares.is_none());
        Ok(())
    }

    #[test]
    fn queens_file_from_empty_board_str() {
        let file_str = "";
        let file_result = QueensFile::from_str(file_str);
        assert!(file_result.is_err());
    }

    #[test]
    fn queens_file_from_invalid_board_str() {
        let file_str = "wwww\nkkkk\nrrrr\nbbbb\n\nQxxx\nxx..\nx...";
        let file_result = QueensFile::from_str(file_str);
        assert!(file_result.is_err());
    }

    #[test]
    fn queens_file_from_board_and_squares_str() -> Result<()> {
        let file_str = "wwww\nkkkk\nrrrr\nbbbb\n\nQxxx\nxx..\nx...\nx...";
        let file = QueensFile::from_str(file_str)?;
        assert_eq!(file.board.size(), 4);
        assert!(file.squares.is_some());
        let squares = file.squares.unwrap().0;
        assert_eq!(
            squares
                .iter()
                .filter(|&&x| x == Some(SquareVal::Queen))
                .collect::<Vec<_>>()
                .len(),
            1
        );
        assert_eq!(
            squares
                .iter()
                .filter(|&&x| x == Some(SquareVal::X))
                .collect::<Vec<_>>()
                .len(),
            7
        );
        Ok(())
    }
}
