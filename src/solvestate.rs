use std::fmt::{Display, Formatter, Write};

use anyhow::{Result, anyhow};
use clap::ValueEnum;
use itertools::{Itertools, Position};
use log::trace;
use owo_colors::{AnsiColors, OwoColorize};

use crate::{
    board::Board,
    datastructure::{Coord, CoordSet},
    file::QueensFile,
    heuristic::Changes,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The assigned state of a square during the solving process.
///
/// As a solver solves a given board, it does so by confirming and eliminating
/// certain squares. This represents the state of those squares -- Queen means
/// it has confirmed it contains a queen, and X means it knows that square
/// cannot contain a queen.
pub enum SquareVal {
    /// The square contains a queen.
    ///
    /// This value is assigned in the solving process if we are sure that
    /// the square contains a queen.
    Queen,
    /// The square cannot contain a queen
    ///
    /// This value is assigned in the solving process if we are sure that
    /// the square cannot contain a queen.
    X,
}

impl SquareVal {
    /// Attempts to convert a character to a `Option<SquareVal>`
    ///
    /// Note that this is not [From] or [TryFrom] because this returns `Result<Option<SquareVal>>`.
    /// That is, this can return either a SquareVal or None or Err, because some chars represent
    /// SquareVals, some represent "blanks", and some are invalid.
    ///
    /// # Examples
    /// ```
    /// use qsolve::solvestate::SquareVal;
    /// assert_eq!(SquareVal::try_from('Q').unwrap(), Some(SquareVal::Queen));
    /// assert_eq!(SquareVal::try_from('x').unwrap(), Some(SquareVal::X));
    /// assert!(SquareVal::try_from('.').unwrap().is_none());
    /// assert!(SquareVal::try_from('S').is_err());
    /// ```
    pub fn try_from(c: char) -> Result<Option<Self>> {
        match c {
            'Q' => Ok(Some(SquareVal::Queen)),
            'x' => Ok(Some(SquareVal::X)),
            ' ' => Ok(None),
            '.' => Ok(None),
            '_' => Ok(None),
            _ => Err(anyhow!("Blank")),
        }
    }
}

impl SquareVal {
    /// Converts the given SquareVal to a character for display.
    ///
    /// Sometimes the square needs to be "highlighted" in some way -- to demonstrate
    /// that the square is under consideration by a heuristic, for example. In general,
    /// we assume the caller will bold the character to do so, but if the square is blank,
    /// a bold space and a space look the same. So this function will oftentimes return a
    /// different character in that case.
    ///
    /// The Charset passed in allows customization of the display.
    pub fn as_char(sv: Option<SquareVal>, highlighted: bool, charset: &Charset) -> char {
        match (sv, highlighted, charset) {
            (Some(SquareVal::Queen), _, Charset::Ascii) => 'Q',
            (Some(SquareVal::Queen), _, Charset::Unicode) => '\u{265B}',
            (Some(SquareVal::X), _, Charset::Ascii) => 'x',
            (Some(SquareVal::X), _, Charset::Unicode) => '\u{00D7}',
            (None, true, Charset::Ascii) => '.',
            (None, true, Charset::Unicode) => '\u{2218}',
            (None, false, _) => ' ',
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
/// What strategy to use for solving the puzzle
pub enum SolveStrategy {
    /// Optimize for generating a solution quickly
    #[default]
    Fast,
    /// Optimize for generating a solution in as few steps as possible
    Short,
    /// Optimize for generating a solution using the simplest moves
    Simple,
}

impl Display for SolveStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
/// What characters to use in the animation
pub enum Charset {
    /// Uses ASCII characters; Q for queens, x for impossible
    Ascii,
    /// Uses Unicode characters; a queen for queens, and a smaller centered
    /// x for impossible.
    #[default]
    Unicode,
}

#[derive(Clone, Debug)]
/// A representation of a board in the process of being solved. This contains
/// a board (which is constant across a given solving process) and a (possibly
/// incomplete) set of markings on the squares of the board, indicating whether
/// a queen is present, impossible, or still to be determined.
///
/// # Invariants
///
/// We always assume that whenever a Queen has been placed, all squares that the
/// Queen eliminates (be that by row, column, color or adjacency) are x'd out.
/// Especially when taking input from the user, it's important to ensure that
/// invariant is followed. The easiest way to do so is by calling
/// [SolveState::apply_changes] once for each Queen.
pub struct SolveState<'a> {
    /// The board that this solve state is solving
    pub board: &'a Board,

    /// A list of the solve state's values for each square.
    ///
    /// This is stored as a 1D vector, where the first N
    /// values are the first row, the next N values the second row,
    /// and so on.
    ///
    /// Each value is an `Option<SquareVal>`, so it can either be:
    ///  * None, meaning blank
    ///  * Some(Queen), meaning we know a queen is there
    ///  * Some(X), meaning we know no queen can be there
    squares: Vec<Option<SquareVal>>,
}

impl<'a> From<&'a QueensFile> for SolveState<'a> {
    fn from(queens_file: &'a QueensFile) -> Self {
        let mut solve_state = SolveState {
            board: &queens_file.board,
            squares: queens_file
                .squares
                .clone()
                .map(|x| x.into())
                .unwrap_or_else(|| vec![None; queens_file.board.square_count()]),
        };

        // So a Queens File might have Queens listed and not have the x's that those
        // Queens imply. This library assumes a SolveState always has those x's in place,
        // so we need to check that here to avoid violating that invariant.

        for (idx, _) in solve_state
            .clone()
            .squares
            .iter()
            .enumerate()
            .filter(|&(_, &sv)| sv == Some(SquareVal::Queen))
        {
            let queen = solve_state.board.idx_to_coord(&idx);
            let x = solve_state
                .board
                .queen_borders(&queen)
                .iter()
                .filter(|&coord| solve_state.square(&coord).is_none())
                .collect::<CoordSet>();
            solve_state.apply_changes(&Changes::AddQueen { queen, x });
        }

        trace!("From<QueensFile> for SolveState done:\n{}", solve_state);

        solve_state
    }
}

impl SolveState<'_> {
    /// Returns whether the board is complete: that is, whether
    /// there are the same number of queens as their are rows/cols/colors.
    pub fn complete(&self) -> bool {
        self.squares
            .iter()
            .filter(|&&x| x == Some(SquareVal::Queen))
            .count()
            == self.board.size()
    }

    /// Returns whether the board is valid.
    ///
    /// This requires:
    /// * No column contains multiple queens.
    /// * No row contains multiple queens.
    /// * No color contains multiple queens.
    /// * No queens border each other.
    pub fn is_valid(&self) -> bool {
        let size = self.board.size();
        let rows_valid = (0..size).all(|r| {
            self.board
                .row_coords(r)
                .iter()
                .map(|c| self.square(&c))
                .filter(|&sv| sv == Some(SquareVal::Queen))
                .count()
                <= 1
        });
        let cols_valid = (0..size).all(|c| {
            self.board
                .col_coords(c)
                .iter()
                .map(|c| self.square(&c))
                .filter(|&sv| sv == Some(SquareVal::Queen))
                .count()
                <= 1
        });
        let colors_valid = self.board.all_colors().iter().all(|&&color| {
            self.board
                .all_coords()
                .iter()
                .filter(|c| self.board.color(c) == color)
                .map(|c| self.square(&c))
                .filter(|&sv| sv == Some(SquareVal::Queen))
                .count()
                <= 1
        });
        let queen_coords = self
            .squares
            .iter()
            .enumerate()
            .filter(|&(_, &square)| square == Some(SquareVal::Queen))
            .map(|(idx, _)| self.board.idx_to_coord(&idx))
            .collect::<CoordSet>();
        let queens_valid = queen_coords.clone().iter().all(|c| {
            self.board
                .queen_borders(&c)
                .intersection(&queen_coords)
                .is_empty()
        });
        rows_valid && cols_valid && colors_valid && queens_valid
    }

    /// Returns the value in the given square.
    pub fn square(&self, coord: &Coord) -> Option<SquareVal> {
        self.squares[self.board.coord_to_idx(coord)]
    }

    /// Applies all of the provided changes, mutating the underlying
    /// SolveState accordingly.
    pub fn apply_changes(&mut self, changes: &Changes) {
        match changes {
            Changes::AddQueen { queen, x } => {
                self.squares[self.board.coord_to_idx(queen)] = Some(SquareVal::Queen);
                for coord in x {
                    self.squares[self.board.coord_to_idx(&coord)] = Some(SquareVal::X)
                }
            }
            Changes::AddX { x } => {
                for coord in x {
                    self.squares[self.board.coord_to_idx(&coord)] = Some(SquareVal::X)
                }
            }
        }
    }

    /// Returns a string colored by OwoColorize that represents the
    /// SolveState, highlighting the given Coordinates.
    pub fn ansi_string(&self, highlight: CoordSet, charset: Charset) -> Result<String> {
        let mut f = String::new();
        for row_num in 0..self.board.size() {
            for col_num in 0..self.board.size() {
                let coord = (row_num, col_num);
                let highlight = highlight.contains(&coord);
                let square = self.square(&coord);
                let ansi_color = AnsiColors::from(self.board.color(&coord));
                let fg_color = self.board.color(&coord).fg_color();
                let c = SquareVal::as_char(square, highlight, &charset);
                if highlight {
                    write!(
                        f,
                        "{}",
                        c.color(fg_color).on_color(ansi_color).bold().underline()
                    )?
                } else {
                    write!(f, "{}", c.color(fg_color).on_color(ansi_color))?
                }
            }
            if row_num != self.board.size() - 1 {
                writeln!(f)?
            };
        }
        Ok(f)
    }
}

impl<'a> From<&'a Board> for SolveState<'a> {
    fn from(board: &'a Board) -> Self {
        let squares = vec![None; board.square_count()];
        SolveState { board, squares }
    }
}

impl Display for SolveState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.board)?;
        writeln!(f)?;
        for (pos, row) in self.squares.chunks_exact(self.board.size()).with_position() {
            for square in row {
                write!(f, "{}", SquareVal::as_char(*square, false, &Charset::Ascii))?;
            }
            if pos != Position::Last {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use regex::Regex;

    use super::*;

    #[test]
    fn squareval_as_char() {
        let vals = [None, Some(SquareVal::Queen), Some(SquareVal::X)];
        for v in vals {
            let ascii_char = SquareVal::as_char(v, false, &Charset::Ascii);
            assert!(ascii_char.is_ascii() || ascii_char.is_whitespace());
            let unicode_char = SquareVal::as_char(v, false, &Charset::Unicode);
            assert!(!unicode_char.is_ascii() || unicode_char.is_whitespace());
            let ascii_highlighted_char = SquareVal::as_char(v, true, &Charset::Ascii);
            assert!(ascii_highlighted_char.is_ascii() || ascii_highlighted_char.is_whitespace());
            let unicode_highlighted_char = SquareVal::as_char(v, true, &Charset::Unicode);
            assert!(
                !unicode_highlighted_char.is_ascii() || unicode_highlighted_char.is_whitespace()
            );
        }
    }

    #[test]
    fn solvestate_from_board() {
        let board = Board::from_str("wwww\nkkkk\nrrrr\nbbbb").unwrap();
        let ss = SolveState::from(&board);
        assert!(ss.squares.iter().all(Option::is_none));
    }

    #[test]
    fn solvestate_from_queens_file() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let squares_str = "Qxxx\nxx..\nx...\nx. _";
        let qf_str = format!("{}\n\n{}", board_str, squares_str);
        let qf = QueensFile::from_str(&qf_str).unwrap();
        let ss = SolveState::from(&qf);
        assert!(ss.is_valid());

        assert_eq!(
            format!("{}", ss),
            qf_str.replace(".", " ").replace("_", " ")
        );
    }

    #[test]
    fn solvestate_ansi_string() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let squares_str = "Qxxx\nxx..\nx...\nx. _";
        let qf_str = format!("{}\n\n{}", board_str, squares_str);
        let qf = QueensFile::from_str(&qf_str).unwrap();
        let ss = SolveState::from(&qf);
        assert!(ss.is_valid());

        let ansi_string = ss.ansi_string(CoordSet::default(), Charset::Ascii).unwrap();
        let ansi_re = Regex::new(r"\u{1b}\[[0-9;]*m").unwrap();
        let ansi_removed = ansi_re.replace_all(&ansi_string, "");
        assert_eq!(
            ansi_removed,
            squares_str.replace(".", " ").replace("_", " ")
        );
    }

    #[test]
    fn solvestate_ansi_string_highlighted() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let squares_str = "Qxxx\nxx..\nx...\nx. _";
        let qf_str = format!("{}\n\n{}", board_str, squares_str);
        let qf = QueensFile::from_str(&qf_str).unwrap();
        let ss = SolveState::from(&qf);
        assert!(ss.is_valid());

        let ansi_string = ss
            .ansi_string(CoordSet::from_iter(vec![(0, 0)]), Charset::Ascii)
            .unwrap();
        let ansi_re = Regex::new(r"\u{1b}\[[0-9;]*m").unwrap();
        let ansi_removed = ansi_re.replace_all(&ansi_string, "");
        assert_eq!(
            ansi_removed,
            squares_str.replace(".", " ").replace("_", " ")
        );
    }

    #[test]
    fn solvestrategy_display() {
        assert_eq!(format!("{}", SolveStrategy::Fast), "Fast");
        assert_eq!(format!("{}", SolveStrategy::Short), "Short");
        assert_eq!(format!("{}", SolveStrategy::Simple), "Simple");
    }
}
