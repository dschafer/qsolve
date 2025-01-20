use itertools::Itertools;
use log::{debug, trace};

use crate::{
    board::Board,
    datastructure::{Coord, CoordSet, LineSet, SquareColorSet},
    solvestate::{SolveState, SolveStrategy, SquareVal},
};

#[derive(Debug, Eq, PartialEq)]
/// Represents a set of changes that a heuristic wants to
/// make to the board.
pub enum Changes {
    /// The heuristic is adding a queen at the given coordinate, and
    /// is x'ing out the provided list of coordinates as now being
    /// impossible.
    AddQueen {
        /// What Coord is the new queen at.
        queen: Coord,
        /// What coords need to be x'd out.
        x: CoordSet,
    },

    /// The heuristic has ruled out the given list of coordinates
    /// as now being impossible.
    AddX {
        /// What coords need to be x'd out.
        x: CoordSet,
    },
}

impl Changes {
    /// Returns all of the coords changed by the given [Changes].
    pub fn changed_coords(&self) -> CoordSet {
        match self {
            Changes::AddQueen { queen, x } => {
                let mut cs = *x;
                cs.add(*queen);
                cs
            }
            Changes::AddX { x } => *x,
        }
    }
}

/// Represents a heuristic for solving a Queens board.
pub trait Heuristic: std::fmt::Debug {
    /// What changes would this heuristic make? This returns None
    /// if the heuristic does not see any possible changes, or
    /// returns Some(Changes) containing the changes it would make.
    fn changes(&self, solve_state: &SolveState) -> Option<Changes>;

    /// What coordinates did this heuristic consider?
    ///
    /// This allows visualizations to highlight appropriate squares, or
    /// solvers to prioritize strategies that "look at" fewer coordinates.
    fn seen_coords(&self, solve_state: &SolveState) -> CoordSet;

    /// A human explanation of what this heuristic does.
    ///
    /// Important: This should always be two lines, so that animations
    /// know where to move the cursor. The recommended format is:
    /// 1. First line -- an explanation of what the heuristic saw.
    /// 2. Second line -- an explanation of what the heuristic does.
    fn description(&self) -> String;
}

/// Returns the next heuristic to use for the given solve state.
///
/// # Invariants
///
/// If this returns `Some(h)`, then `h.changes(solve_state)` will always
/// return `Some` and not `None`.
pub fn next_heuristic<'h>(
    solve_state: &SolveState<'_>,
    solve_strategy: SolveStrategy,
    heuristics: &'h [Box<dyn Heuristic>],
) -> Option<&'h dyn Heuristic> {
    debug!(
        "Generating next heuristic with {:?} strategy",
        solve_strategy
    );
    let h = match solve_strategy {
        SolveStrategy::Short => heuristics
            .iter()
            .filter(|&h| h.changes(solve_state).is_some())
            .max_by_key(|&h| {
                let changes = h.changes(solve_state);
                match changes {
                    Some(Changes::AddQueen { queen: _, x }) => (1, x.len()),
                    Some(Changes::AddX { x }) => (0, x.len()),
                    None => (0, 0),
                }
            }),
        SolveStrategy::Simple => heuristics
            .iter()
            .filter(|&h| h.changes(solve_state).is_some())
            .max_by_key(|&h| {
                let seen = h.seen_coords(solve_state);
                let changes = h.changes(solve_state);
                match changes {
                    Some(Changes::AddQueen { queen: _, x }) => (
                        1,
                        1 + solve_state.board.square_count() - seen.len(),
                        x.len(),
                    ),
                    Some(Changes::AddX { x }) => (
                        0,
                        1 + solve_state.board.square_count() - seen.len(),
                        x.len(),
                    ),
                    None => (0, 0, 0),
                }
            }),
        SolveStrategy::Fast => heuristics
            .iter()
            .find(|&h| h.changes(solve_state).is_some()),
    };
    // Needed for box manipulation
    h.map(|v| &**v)
}

/// Returns a list of all available heuristics for the given board
pub fn all_heuristics(board: &Board) -> Vec<Box<dyn Heuristic>> {
    debug!("Heuristic generation started.");
    let mut v: Vec<Box<dyn Heuristic>> = vec![];
    v.extend(board.all_colors().iter().map(|color| {
        Box::new(LastSquareAvailable {
            coords: board.coords_for_color(color),
            desc: format!("'{:?}' Color", color),
        }) as _
    }));
    v.extend((0..board.size()).map(|r| {
        Box::new(LastSquareAvailable {
            coords: board.row_coords(r),
            desc: format!("Row {}", r + 1),
        }) as _
    }));
    v.extend((0..board.size()).map(|c| {
        Box::new(LastSquareAvailable {
            coords: board.col_coords(c),
            desc: format!("Col {}", c + 1),
        }) as _
    }));
    v.extend((0..board.size()).map(|r| {
        Box::new(AllPossibilitiesEliminateSquare {
            coords: board.row_coords(r),
            desc: format!("Row {}", r + 1),
        }) as _
    }));
    v.extend((0..board.size()).map(|c| {
        Box::new(AllPossibilitiesEliminateSquare {
            coords: board.col_coords(c),
            desc: format!("Col {}", c + 1),
        }) as _
    }));
    v.extend(board.all_colors().into_iter().map(|color| {
        Box::new(AllPossibilitiesEliminateSquare {
            coords: board.coords_for_color(color),
            desc: format!("'{:?}' Color", color),
        }) as _
    }));

    v.extend(
        (0..board.size())
            .powerset()
            .filter(|ll| !ll.is_empty() && ll.len() <= board.size() / 2)
            .flat_map(|ll| {
                vec![
                    Box::new(NLinesContainOnlyNColors {
                        lines: ll.iter().map(|&l| board.col_coords(l)).collect(),
                        desc: format!("Cols {:?}", ll.iter().map(|x| x + 1).collect::<Vec<_>>()),
                    }) as _,
                    Box::new(NLinesContainOnlyNColors {
                        lines: ll.iter().map(|&l| board.row_coords(l)).collect(),
                        desc: format!("Rows {:?}", ll.iter().map(|x| x + 1).collect::<Vec<_>>()),
                    }) as _,
                ]
            }),
    );
    v.extend(
        board
            .all_colors()
            .into_iter()
            .powerset()
            .filter(|cc| !cc.is_empty())
            .map(|cc| {
                Box::new(NColorsOnlyAppearInNLines {
                    color_desc: format!("{:?}", cc),
                    colors: SquareColorSet::from_iter(cc.into_iter().copied()),
                    liner: |coord| coord.0,
                    liner_desc: "rows".to_string(),
                }) as _
            }),
    );
    v.extend(
        board
            .all_colors()
            .into_iter()
            .powerset()
            .filter(|cc| !cc.is_empty())
            .map(|cc| {
                Box::new(NColorsOnlyAppearInNLines {
                    color_desc: format!("{:?}", cc),
                    colors: SquareColorSet::from_iter(cc.into_iter().copied()),
                    liner: |coord| coord.1,
                    liner_desc: "cols".to_string(),
                }) as _
            }),
    );
    debug!("Heuristic generation completed.");
    v
}

#[derive(Debug)]
struct LastSquareAvailable {
    coords: CoordSet,
    desc: String,
}

impl Heuristic for LastSquareAvailable {
    fn seen_coords(&self, _solve_state: &SolveState) -> CoordSet {
        self.coords
    }
    fn changes(&self, solve_state: &SolveState) -> Option<Changes> {
        trace!("Heuristic Start: LastSquareAvailable {:?}", self);
        let last_empty_coord = self
            .coords
            .iter()
            .filter(|&c| solve_state.square(&c).is_none())
            .exactly_one()
            .ok();
        let queen = last_empty_coord?;
        trace!("Heuristic Success: LastSquareAvailable {:?}", self);
        let x = solve_state
            .board
            .queen_borders(&queen)
            .iter()
            .filter(|&coord| solve_state.square(&coord).is_none())
            .collect::<CoordSet>();
        trace!("Heuristic Return: LastSquareAvailable {:?}", self);
        Some(Changes::AddQueen { queen, x })
    }

    fn description(&self) -> String {
        format!(
            "There is only one possiblity left for {}.\nFill that in with a Queen (and x out new impossibilities)",
            self.desc
        )
    }
}

#[derive(Debug)]
struct AllPossibilitiesEliminateSquare {
    coords: CoordSet,
    desc: String,
}

impl Heuristic for AllPossibilitiesEliminateSquare {
    fn seen_coords(&self, solve_state: &SolveState) -> CoordSet {
        self.coords
            .iter()
            .filter(|&coord| solve_state.square(&coord).is_none())
            .collect()
    }
    fn changes(&self, solve_state: &SolveState) -> Option<Changes> {
        trace!(
            "Heuristic Start: AllPossibilitiesEliminateSquare {:?}",
            self
        );
        let x = self
            .coords
            .iter()
            .filter(|&coord| solve_state.square(&coord).is_none())
            .map(|coord| solve_state.board.queen_borders(&coord))
            .reduce(|acc, e| acc.intersection(&e))
            .unwrap_or_default()
            .iter()
            .filter(|coord| solve_state.square(coord).is_none())
            .collect::<CoordSet>();
        if x.is_empty() {
            None
        } else {
            trace!(
                "Heuristic Success/Return: AllPossibilitiesEliminateSquare {:?}",
                self
            );
            Some(Changes::AddX { x })
        }
    }

    fn description(&self) -> String {
        format!(
            "All of the possible queens for {} eliminate certain squares.\nx out those squares.",
            self.desc
        )
    }
}

#[derive(Debug)]
struct NLinesContainOnlyNColors {
    lines: Vec<CoordSet>,
    desc: String,
}

impl Heuristic for NLinesContainOnlyNColors {
    fn seen_coords(&self, solve_state: &SolveState) -> CoordSet {
        self.lines
            .iter()
            .flatten()
            .filter(|&coord| solve_state.square(&coord).is_none())
            .collect()
    }
    fn changes(&self, solve_state: &SolveState) -> Option<Changes> {
        trace!("Heuristic Start: NLinesContainOnlyNColors {:?}", self);
        if self
            .lines
            .iter()
            .flatten()
            .any(|coord| solve_state.square(&coord) == Some(SquareVal::Queen))
        {
            trace!("Heuristic Invalid: NLinesContainOnlyNColors {:?}", self);
            return None;
        }
        let coords = CoordSet::from_iter(
            self.lines
                .iter()
                .flatten()
                .filter(|&coord| solve_state.square(&coord).is_none()),
        );
        let colors_set =
            SquareColorSet::from_iter(coords.iter().map(|coord| solve_state.board.color(&coord)));
        if colors_set.len() > self.lines.len() {
            trace!("Heuristic Invalid: NLinesContainOnlyNColors {:?}", self);
            return None;
        }
        trace!("Heuristic Success: NLinesContainOnlyNColors {:?}", self);
        let x = solve_state
            .board
            .all_coords()
            .iter()
            .filter(|coord| colors_set.contains(&solve_state.board.color(coord)))
            .filter(|coord| !coords.contains(coord))
            .filter(|coord| solve_state.square(coord).is_none())
            .collect::<CoordSet>();
        if x.is_empty() {
            trace!("Heuristic No-op: NLinesContainOnlyNColors {:?}", self);
            None
        } else {
            trace!("Heuristic Return: NLinesContainOnlyNColors {:?}", self);
            Some(Changes::AddX { x })
        }
    }

    fn description(&self) -> String {
        format!(
            "There are only {} remaining colors on {}.\nx out all other instances of those colors",
            self.lines.len(),
            self.desc,
        )
    }
}

#[derive(Debug)]
struct NColorsOnlyAppearInNLines {
    colors: SquareColorSet,
    liner: fn(Coord) -> usize,
    color_desc: String,
    liner_desc: String,
}

impl Heuristic for NColorsOnlyAppearInNLines {
    fn seen_coords(&self, solve_state: &SolveState) -> CoordSet {
        solve_state
            .board
            .all_coords()
            .iter()
            .filter(|coord| self.colors.contains(&solve_state.board.color(coord)))
            .filter(|coord| solve_state.square(coord).is_none())
            .collect()
    }
    fn changes(&self, solve_state: &SolveState) -> Option<Changes> {
        trace!("Heuristic Start: NLinesContainOnlyNColors {:?}", self);
        if solve_state
            .board
            .all_coords()
            .iter()
            .filter(|coord| self.colors.contains(&solve_state.board.color(coord)))
            .any(|coord| solve_state.square(&coord) == Some(SquareVal::Queen))
        {
            trace!("Heuristic Invalid: NLinesContainOnlyNColors {:?}", self);
            return None;
        }
        let coords = solve_state.board.all_coords();
        let lines = coords
            .iter()
            .filter(|coord| self.colors.contains(&solve_state.board.color(coord)))
            .filter(|coord| solve_state.square(coord).is_none())
            .map(self.liner);
        let lines_set = LineSet::from_iter(lines);
        if lines_set.len() > self.colors.len() {
            trace!("Heuristic Invalid: NLinesContainOnlyNColors {:?}", self);
            return None;
        }
        trace!("Heuristic Success: NLinesContainOnlyNColors {:?}", self);
        let x = solve_state
            .board
            .all_coords()
            .iter()
            .filter(|&coord| lines_set.contains(&(self.liner)(coord)))
            .filter(|coord| !self.colors.contains(&solve_state.board.color(coord)))
            .filter(|coord| solve_state.square(coord).is_none())
            .collect::<CoordSet>();
        if x.is_empty() {
            trace!("Heuristic No-op: NLinesContainOnlyNColors {:?}", self);
            None
        } else {
            trace!("Heuristic Return: NLinesContainOnlyNColors {:?}", self);
            Some(Changes::AddX { x })
        }
    }

    fn description(&self) -> String {
        format!(
            "{} appear on only {} {}.\nx out all other colors that appear on those {}",
            self.color_desc,
            self.colors.len(),
            self.liner_desc,
            self.liner_desc
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;

    use crate::{file::QueensFile, squarecolor::SquareColor};

    use super::*;

    #[test]
    fn test_changed_coords() {
        let x = CoordSet::from_iter(vec![(0, 0), (1, 1)]);
        assert_eq!(Changes::AddX { x }.changed_coords(), x);
        assert_eq!(
            Changes::AddQueen { queen: (0, 2), x }.changed_coords(),
            CoordSet::from_iter(x.iter().chain(vec![(0, 2)]))
        );
    }

    #[test]
    fn last_square_available() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\nx...\n....\nx...\nx...";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = LastSquareAvailable {
            coords: ss.board.col_coords(0),
            desc: String::new(),
        };
        assert_eq!(
            heuristic.changes(&ss),
            Some(Changes::AddQueen {
                queen: (1, 0),
                x: CoordSet::from_iter(vec![
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    (2, 1),
                    (1, 1),
                    (1, 2),
                    (1, 3)
                ])
            })
        );
        assert_eq!(heuristic.seen_coords(&ss), ss.board.col_coords(0));
        Ok(())
    }

    #[test]
    fn last_square_available_ignores_complete_cols() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\nx...\nQ...\nx...\nx...";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = LastSquareAvailable {
            coords: ss.board.col_coords(0),
            desc: String::new(),
        };
        assert_eq!(heuristic.changes(&ss), None);
        assert_eq!(heuristic.seen_coords(&ss), ss.board.col_coords(0));
        Ok(())
    }

    #[test]
    fn last_square_available_ignores_cols_with_blanks() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\nx...\nx...";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = LastSquareAvailable {
            coords: ss.board.col_coords(0),
            desc: String::new(),
        };
        assert_eq!(heuristic.changes(&ss), None);
        assert_eq!(heuristic.seen_coords(&ss), ss.board.col_coords(0));
        Ok(())
    }

    #[test]
    fn last_square_available_description() {
        let heuristic = LastSquareAvailable {
            coords: CoordSet::default(),
            desc: "desc".to_string(),
        };
        assert!(heuristic.description().contains("desc"));
    }

    #[test]
    fn all_possibilities_eliminate_square() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = AllPossibilitiesEliminateSquare {
            coords: ss.board.coords_for_color(&SquareColor::Black),
            desc: String::new(),
        };
        assert_eq!(
            heuristic.changes(&ss),
            Some(Changes::AddX {
                x: CoordSet::from_iter(vec![(1, 0), (0, 2), (2, 2)])
            })
        );
        assert_eq!(
            heuristic.seen_coords(&ss),
            ss.board.coords_for_color(&SquareColor::Black)
        );
        Ok(())
    }

    #[test]
    fn all_possibilities_eliminate_square_with_xs() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n.x..\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = AllPossibilitiesEliminateSquare {
            coords: ss.board.coords_for_color(&SquareColor::Black),
            desc: String::new(),
        };
        assert_eq!(
            heuristic.changes(&ss),
            Some(Changes::AddX {
                x: CoordSet::from_iter(vec![(1, 0), (0, 2), (0, 3), (2, 2), (2, 3)])
            })
        );
        assert_eq!(
            heuristic.seen_coords(&ss),
            CoordSet::from_iter(vec![(1, 2), (1, 3)])
        );
        Ok(())
    }

    #[test]
    fn all_possibilities_intersects_to_zeros() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = AllPossibilitiesEliminateSquare {
            coords: ss.board.coords_for_color(&SquareColor::Cyan),
            desc: String::new(),
        };
        assert_eq!(heuristic.changes(&ss), None);
        assert_eq!(
            heuristic.seen_coords(&ss),
            CoordSet::from_iter(vec![(3, 0), (3, 1), (3, 2), (3, 3)])
        );
        Ok(())
    }

    #[test]
    fn all_possibilities_description() {
        let heuristic = AllPossibilitiesEliminateSquare {
            coords: CoordSet::default(),
            desc: "desc".to_string(),
        };
        assert!(heuristic.description().contains("desc"));
    }

    #[test]
    fn nlines_contain_only_ncolors() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = NLinesContainOnlyNColors {
            lines: vec![ss.board.row_coords(0)],
            desc: String::new(),
        };
        assert_eq!(
            heuristic.changes(&ss),
            Some(Changes::AddX {
                x: CoordSet::from_iter(vec![(1, 0)])
            })
        );
        assert_eq!(heuristic.seen_coords(&ss), ss.board.row_coords(0));
        Ok(())
    }

    #[test]
    fn nlines_contain_only_ncolors_fails() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = NLinesContainOnlyNColors {
            lines: vec![ss.board.row_coords(1)],
            desc: String::new(),
        };
        assert_eq!(heuristic.changes(&ss), None);
        assert_eq!(heuristic.seen_coords(&ss), ss.board.row_coords(1));
        Ok(())
    }

    #[test]
    fn nlines_contain_only_ncolors_description() {
        let heuristic = NLinesContainOnlyNColors {
            lines: vec![],
            desc: "desc".to_string(),
        };
        assert!(heuristic.description().contains("desc"));
    }

    #[test]
    fn ncolors_only_appear_in_nlines() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = NColorsOnlyAppearInNLines {
            colors: SquareColorSet::from_iter([
                SquareColor::Black,
                SquareColor::Blue,
                SquareColor::Cyan,
            ]),
            liner: |coord| coord.0,
            color_desc: String::new(),
            liner_desc: String::new(),
        };
        assert_eq!(
            heuristic.changes(&ss),
            Some(Changes::AddX {
                x: CoordSet::from_iter(vec![(1, 0)])
            })
        );
        assert_eq!(
            heuristic.seen_coords(&ss),
            CoordSet::from_iter(
                ss.board
                    .coords_for_color(&SquareColor::Black)
                    .iter()
                    .chain(ss.board.coords_for_color(&SquareColor::Blue).iter())
                    .chain(ss.board.coords_for_color(&SquareColor::Cyan).iter())
            )
        );
        Ok(())
    }

    #[test]
    fn ncolors_only_appear_in_nlines_fails() -> Result<()> {
        let input_str = "rrrr\nrkkk\nbbbb\ncccc\n\n....\n....\n....\n....";
        let queens_file = QueensFile::from_str(input_str)?;
        let ss = SolveState::from(&queens_file);
        assert!(ss.is_valid());
        let heuristic = NColorsOnlyAppearInNLines {
            colors: SquareColorSet::from_iter([SquareColor::Blue, SquareColor::Cyan]),
            liner: |coord| coord.0,
            color_desc: String::new(),
            liner_desc: String::new(),
        };
        assert_eq!(heuristic.changes(&ss), None);
        assert_eq!(
            heuristic.seen_coords(&ss),
            CoordSet::from_iter(
                ss.board
                    .coords_for_color(&SquareColor::Blue)
                    .iter()
                    .chain(ss.board.coords_for_color(&SquareColor::Cyan).iter())
            )
        );
        Ok(())
    }

    #[test]
    fn ncolors_only_appear_in_nlines_description() {
        let heuristic = NColorsOnlyAppearInNLines {
            colors: SquareColorSet::default(),
            liner: |_| 0,
            color_desc: "color".to_string(),
            liner_desc: "liner".to_string(),
        };
        assert!(heuristic.description().contains("color"));
        assert!(heuristic.description().contains("liner"));
    }
}
