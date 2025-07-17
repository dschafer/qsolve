use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::{Result, anyhow};
use itertools::{Itertools, Position, iproduct};

use crate::{
    datastructure::{Coord, CoordSet},
    squarecolor::SquareColor,
};

/// A representation of a Queens board.
///
/// This represents the underlying board on which every game takes place; notably,
/// it does **not** represent the progress that a player is making in solving the
/// board. For that, see [SolveState][crate::solvestate::SolveState].
///
/// # Representation
///
/// The board represents the underlying colors as a 1-dimensional [Vec], in "row-major" form.
/// Most concretely, a board that looks like this:
///
/// ```text
/// kkkk
/// krrr
/// gggg
/// bbbb
/// ```
///
/// would be represented as this [Vec]:
///
/// ```text
/// kkkkkrrrggggbbbb
/// ```
///
/// # Creation
///
/// To create a board, there are two main possiblites.
///
/// One is that the board can be directly constructed from a list of colors, along
/// with the associated size, by using [Board::new].
///
/// For example:
///
/// ```
/// # use qsolve::board::Board;
/// # use qsolve::squarecolor::SquareColor;
/// let board = Board::new(4, vec![SquareColor::Black; 16]);
/// ```
///
/// Alternately, it can be constructed by parsing a string that uses the character abbreviations
/// for [SquareColor]s defined by [SquareColor::try_from], by using [Board::from_str].
///
/// For example:
///
/// ```
/// # use qsolve::board::Board;
/// # use std::str::FromStr;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Board {
    size: usize,
    colors: Vec<SquareColor>,
    coords: CoordSet,
    queen_borders: Vec<CoordSet>,
}

impl Board {
    /// Creates a new Board with the given size and [SquareColor]s.
    ///
    /// # Arguments
    /// * `size` - The size of the square board
    /// * `colors` - A [Vec] of [SquareColor]s for each square in row-major order
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// ```
    pub fn new(size: usize, colors: Vec<SquareColor>) -> Self {
        assert_eq!(
            size * size,
            colors.len(),
            "Colors must be equal to size*size"
        );
        let coords = iproduct!(0..size, 0..size).collect::<CoordSet>();
        let mut board = Board {
            size,
            colors,
            coords,
            queen_borders: vec![],
        };
        board.compute_queen_borders();
        board
    }

    /// Returns the length/width of the board.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.size(), 4);
    /// ```
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the total number of squares on the board.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.square_count(), 16);
    /// ```
    pub fn square_count(&self) -> usize {
        self.size * self.size
    }

    /// Converts a [Coord] to its index in a 1D, row-major representation.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.coord_to_idx(&(0,0)), 0);
    /// assert_eq!(board.coord_to_idx(&(0,1)), 1);
    /// assert_eq!(board.coord_to_idx(&(1,0)), 4);
    /// ```
    pub fn coord_to_idx(&self, coord: &Coord) -> usize {
        coord.0 * self.size + coord.1
    }

    /// Converts an index from a 1D, row-major representation to a [Coord].
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.idx_to_coord(&0), (0,0));
    /// assert_eq!(board.idx_to_coord(&1), (0,1));
    /// assert_eq!(board.idx_to_coord(&4), (1,0));
    /// ```
    pub fn idx_to_coord(&self, idx: &usize) -> Coord {
        (idx / self.size, idx % self.size)
    }

    /// Returns the [SquareColor] for the given [Coord].
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// # use std::str::FromStr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
    /// assert_eq!(board.color(&(0,0)), SquareColor::Black);
    /// # Ok(())
    /// # }
    /// ```
    pub fn color(&self, coord: &Coord) -> SquareColor {
        self.colors[self.coord_to_idx(coord)]
    }

    /// Returns a list of all unique [SquareColor]s in the grid.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// # use std::str::FromStr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
    /// assert_eq!(board.all_colors(), vec![&SquareColor::Black, &SquareColor::Red, &SquareColor::Blue, &SquareColor::White]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn all_colors(&self) -> Vec<&SquareColor> {
        self.colors.iter().unique().collect()
    }

    /// Returns a list of all [Coord]s that contain a given [SquareColor].
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// # use qsolve::datastructure::CoordSet;
    /// # use std::str::FromStr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
    /// assert_eq!(board.coords_for_color(&SquareColor::Red), CoordSet::from_iter(vec![(1,1),(1,2),(1,3)]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn coords_for_color(&self, color: &SquareColor) -> CoordSet {
        self.all_coords()
            .iter()
            .filter(|&coord| self.color(&coord) == *color)
            .collect()
    }

    /// Returns a list of all [Coord]s in the grid.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::datastructure::CoordSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.all_coords().len(), 16);
    /// ```
    pub fn all_coords(&self) -> &CoordSet {
        &self.coords
    }

    /// Returns a list of all [Coord]s in a given row.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::datastructure::CoordSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.row_coords(1), CoordSet::from_iter(vec![(1,0),(1,1),(1,2),(1,3)]));
    /// ```
    pub fn row_coords(&self, r: usize) -> CoordSet {
        (0..self.size).map(|c| (r, c)).collect()
    }

    /// Returns a list of all [Coord]s in a given column.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::datastructure::CoordSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let board = Board::new(4, vec![SquareColor::Black; 16]);
    /// assert_eq!(board.col_coords(1), CoordSet::from_iter(vec![(0,1),(1,1),(2,1),(3,1)]));
    /// ```
    pub fn col_coords(&self, c: usize) -> CoordSet {
        (0..self.size).map(|r| (r, c)).collect()
    }

    /// Returns a set of all [Coord]s that are eliminated (by row, col, color or proximity)
    /// if a queen is placed in the given square.
    ///
    /// # Performance
    ///
    /// This is such a common operation that we pre-compute all borders upon creation
    /// of the board, and this just returns that pre-computed value. So there is no need
    /// to memoize this in an external caller; it is memoized internally.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use qsolve::squarecolor::SquareColor;
    /// # use qsolve::datastructure::CoordSet;
    /// # use std::str::FromStr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
    /// assert_eq!(board.queen_borders(&(0,3)), CoordSet::from_iter(vec![(0,0),(0,1),(0,2),(1,3),(2,3),(3,3),(1,0),(1,2)]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn queen_borders(&self, queen: &Coord) -> CoordSet {
        self.queen_borders[self.coord_to_idx(queen)]
    }

    /// Pre-computes the queen borders to avoid repeating that computation on\
    /// repeated calls to [Board::queen_borders].
    fn compute_queen_borders(&mut self) {
        let mut queen_borders = Vec::with_capacity(self.square_count());
        for idx in 0..self.square_count() {
            let queen = &self.idx_to_coord(&idx);
            let mut hs = CoordSet::default();
            hs.extend(
                (0..self.size)
                    .map(|r| (r, queen.1))
                    .filter(|coord| coord != queen),
            );
            hs.extend(
                (0..self.size)
                    .map(|c| (queen.0, c))
                    .filter(|coord| coord != queen),
            );
            hs.extend(
                self.all_coords()
                    .iter()
                    .filter(|coord| self.color(coord) == self.color(queen))
                    .filter(|coord| coord != queen),
            );
            if queen.0 > 0 && queen.1 > 0 {
                hs.add((queen.0 - 1, queen.1 - 1));
            }
            if queen.0 > 0 && queen.1 < self.size - 1 {
                hs.add((queen.0 - 1, queen.1 + 1));
            }
            if queen.0 < self.size - 1 && queen.1 > 0 {
                hs.add((queen.0 + 1, queen.1 - 1));
            }
            if queen.0 < self.size - 1 && queen.1 < self.size - 1 {
                hs.add((queen.0 + 1, queen.1 + 1));
            }
            queen_borders.push(hs);
        }
        self.queen_borders = queen_borders;
    }
}

impl FromStr for Board {
    type Err = anyhow::Error;

    /// Attempts to parse the given string into a valid board.
    ///
    /// For a string to represent a board, it must have `n` lines, each
    /// with `n` characters. Each character must be convertable to a
    /// [SquareColor] using [SquareColor::try_from].
    ///
    /// # Examples
    /// ```
    /// # use qsolve::board::Board;
    /// # use std::str::FromStr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let board = Board::from_str("kkkk\nkrrr\nbbbb\nwwww")?;
    /// assert_eq!(board.size(), 4);
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        let lines = s.trim().lines().collect::<Vec<_>>();
        let size = lines.len();
        for (line_num, &line) in lines.iter().enumerate() {
            if line.len() != size {
                let row_num = line_num + 1;
                let row_len = line.len();
                return Err(anyhow!(
                    "Invalid board: row {row_num} has {row_len} entries but the board is {size} rows long."
                ));
            }
        }
        let colors = lines
            .into_iter()
            .flat_map(|s| s.chars())
            .map(SquareColor::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Board::new(size, colors))
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (pos, row) in self.colors.chunks_exact(self.size).with_position() {
            write!(
                f,
                "{}",
                row.iter().map(|c| c.to_string()).collect::<String>()
            )?;
            if pos != Position::Last {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_new() {
        let board = Board::new(4, vec![SquareColor::Black; 16]);
        assert_eq!(board.size(), 4);
        assert_eq!(board.square_count(), 16);
    }

    #[test]
    #[should_panic]
    fn board_wrong_size() {
        Board::new(4, vec![SquareColor::Black; 15]);
    }

    #[test]
    fn board_from_str() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let board_result = Board::from_str(board_str);
        assert!(board_result.is_ok());
        let board = board_result.unwrap();
        assert_eq!(board.size(), 4);
        assert_eq!(board.square_count(), 16);
        assert_eq!(format!("{board}"), board_str);
    }

    #[test]
    fn board_from_invalid_size_str() {
        let board_str = "wwwwwwwwwww\nkkkk\nrrrr\nbbbb";
        let board_result = Board::from_str(board_str);
        assert!(board_result.is_err());
    }

    #[test]
    fn board_from_invalid_char_str() {
        let board_str = "xxxx\nkkkk\nrrrr\nbbbb";
        let board_result = Board::from_str(board_str);
        assert!(board_result.is_err());
    }

    #[test]
    fn board_coord_idx() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let board = Board::from_str(board_str).unwrap();
        assert_eq!(board.size(), 4);
        assert_eq!(board.square_count(), 16);
        assert_eq!(board.coord_to_idx(&(1, 1)), 5);
        assert_eq!(board.idx_to_coord(&5), (1, 1));
    }

    #[test]
    fn board_colors() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let board = Board::from_str(board_str).unwrap();

        assert_eq!(board.color(&(1, 1)), SquareColor::Black);
        assert_eq!(
            board.all_colors(),
            vec![
                &SquareColor::White,
                &SquareColor::Black,
                &SquareColor::Red,
                &SquareColor::Blue
            ]
        );
        assert_eq!(
            board.coords_for_color(&SquareColor::Black),
            CoordSet::from_iter(vec![&(1, 0), &(1, 1), &(1, 2), &(1, 3)])
        );
    }

    #[test]
    fn board_rowcol() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let board = Board::from_str(board_str).unwrap();

        assert_eq!(
            board.all_coords(),
            &CoordSet::from_iter(vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 1),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
                (3, 0),
                (3, 1),
                (3, 2),
                (3, 3)
            ])
        );
        assert_eq!(
            board.row_coords(1),
            CoordSet::from_iter(vec![(1, 0), (1, 1), (1, 2), (1, 3)])
        );
        assert_eq!(
            board.col_coords(1),
            CoordSet::from_iter(vec![(0, 1), (1, 1), (2, 1), (3, 1)])
        );
    }

    #[test]
    fn board_queen_borders() {
        let board_str = "wwww\nkkkk\nrrrr\nbbbb";
        let board = Board::from_str(board_str).unwrap();

        let queen_borders = board.queen_borders(&(0, 0));
        assert_eq!(
            queen_borders,
            CoordSet::from_iter(vec![(0, 1), (0, 2), (0, 3), (1, 0), (1, 1), (2, 0), (3, 0)])
        );
    }
}
