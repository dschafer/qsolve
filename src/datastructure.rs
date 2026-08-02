use std::fmt::Display;

use itertools::Itertools;

use crate::squarecolor::{ALL_SQUARE_COLORS, SquareColor};

type SetBits = u16;

/// The maximum number of rows, columns, or colors supported by the bitset-backed data structures.
pub const MAX_SET_SIZE: usize = SetBits::BITS as usize;

fn assert_line_in_bounds(line: usize) {
    assert!(
        line < MAX_SET_SIZE,
        "line index {line} is out of range for LineSet; expected 0..{MAX_SET_SIZE}"
    );
}

fn assert_coord_in_bounds(coord: Coord) {
    assert!(
        coord.0 < MAX_SET_SIZE && coord.1 < MAX_SET_SIZE,
        "coordinate {coord:?} is out of range for CoordSet; each component must be in 0..{MAX_SET_SIZE}"
    );
}

/// A 0-indexed representation of a coordinate on a [Board][crate::board::Board].
///
/// The first element of the tuple represents the row; the
/// second element of the tuple represents the column. This
/// is zero indexed and begins from the upper left.
///
/// So in a board of size N, `(0, 0)` represents the upper left
/// corner, `(0, N-1)` represents the upper right corner, `(N-1, 0)`
/// represents the lower left corner, and `(N-1, N-1)` represents
/// the lower right corner.
pub type Coord = (usize, usize);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// An efficient implementation of a set for SquareColor.
///
/// # Design
///
/// Each color maps to one bit in an integer bitfield.
///
/// This is faster than using the bitvec package based on testing.
pub struct SquareColorSet(SetBits);

impl Display for SquareColorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            ALL_SQUARE_COLORS
                .iter()
                .filter(|&c| self.contains(c))
                .collect::<Vec<_>>()
        )
    }
}

impl FromIterator<SquareColor> for SquareColorSet {
    fn from_iter<T: IntoIterator<Item = SquareColor>>(iter: T) -> Self {
        let mut scs = 0;
        for sc in iter {
            scs |= 1 << (sc as usize)
        }
        SquareColorSet(scs)
    }
}

impl SquareColorSet {
    /// The number of elements in the set.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::SquareColorSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let scs = SquareColorSet::from_iter(vec![SquareColor::Black, SquareColor::Black, SquareColor::Red]);
    /// assert_eq!(scs.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Whether the set is empty.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::SquareColorSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let scs = SquareColorSet::from_iter(vec![SquareColor::Black, SquareColor::Black, SquareColor::Red]);
    /// assert!(!scs.is_empty());
    ///
    /// let scs2 = SquareColorSet::from_iter::<Vec<SquareColor>>(vec![]);
    /// assert!(scs2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Tests whether the set contains a given color.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::SquareColorSet;
    /// # use qsolve::squarecolor::SquareColor;
    /// let scs = SquareColorSet::from_iter(vec![SquareColor::Black, SquareColor::Black, SquareColor::Red]);
    /// assert!(scs.contains(&SquareColor::Black));
    /// assert!(!scs.contains(&SquareColor::White));
    /// ```
    pub fn contains(&self, color: &SquareColor) -> bool {
        ((self.0 >> (*color as usize)) & 1) == 1
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// An efficient implementation of a set for lines.
///
/// This is a shared implementation for either rows or columns ("lines"),
/// since they require identical logic.
///
/// # Design
///
/// Each line maps to one bit in an integer bitfield.
///
/// This is faster than using the bitvec package based on testing.
///
/// Operations panic when passed a line outside `0..MAX_SET_SIZE`.
pub struct LineSet(SetBits);

impl Display for LineSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            (0..MAX_SET_SIZE)
                .filter(|c| self.contains(c))
                .collect::<Vec<_>>()
        )
    }
}

impl FromIterator<usize> for LineSet {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut bits = 0;
        for line in iter {
            assert_line_in_bounds(line);
            bits |= 1 << line;
        }
        LineSet(bits)
    }
}

impl LineSet {
    /// The number of elements in the set.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::LineSet;
    /// let ls = LineSet::from_iter(vec![1, 1, 2]);
    /// assert_eq!(ls.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Whether the set is empty.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::LineSet;
    /// let ls = LineSet::from_iter(vec![1, 1, 2]);
    /// assert!(!ls.is_empty());
    ///
    /// let ls2 = LineSet::from_iter::<Vec<usize>>(vec![]);
    /// assert!(ls2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Tests whether the set contains a given line.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::LineSet;
    /// let ls = LineSet::from_iter(vec![1, 1, 2]);
    /// assert!(ls.contains(&1));
    /// assert!(!ls.contains(&3));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when `line` is outside `0..MAX_SET_SIZE`.
    pub fn contains(&self, line: &usize) -> bool {
        assert_line_in_bounds(*line);
        ((self.0 >> *line) & 1) == 1
    }

    /// Returns an [Iterator] over the LineSet.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::LineSet;
    /// let ls = LineSet::from_iter(vec![1, 1, 2,5]);
    /// let small = ls.iter().filter(|&a| a <= 2).collect::<LineSet>();
    /// assert_eq!(small, LineSet::from_iter(vec![1,2]))
    /// ```
    pub fn iter(&self) -> LineSetIter<'_> {
        LineSetIter {
            line_set: self,
            idx: 0,
        }
    }
}

/// An iterator over [LineSet].
pub struct LineSetIter<'a> {
    line_set: &'a LineSet,
    idx: usize,
}

impl Iterator for LineSetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < MAX_SET_SIZE {
            if self.line_set.contains(&self.idx) {
                self.idx += 1;
                return Some(self.idx - 1);
            }
            self.idx += 1;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(MAX_SET_SIZE - self.idx))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// An efficient implementation of a set for coords.
///
/// Each row has an integer bitfield with one bit per column.
///
/// This is faster than using the bitvec package based on testing.
///
/// Operations panic when either coordinate component is outside `0..MAX_SET_SIZE`.
pub struct CoordSet([SetBits; MAX_SET_SIZE]);

impl Display for CoordSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            (0..MAX_SET_SIZE)
                .cartesian_product(0..MAX_SET_SIZE)
                .filter(|c| self.contains(c))
                .collect::<Vec<_>>()
        )
    }
}

impl<'a> FromIterator<&'a Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = &'a Coord>>(iter: T) -> Self {
        let mut coord_set = CoordSet::default();
        for coord in iter {
            coord_set.add(*coord);
        }
        coord_set
    }
}

impl FromIterator<Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = Coord>>(iter: T) -> Self {
        let mut coord_set = CoordSet::default();
        for coord in iter {
            coord_set.add(coord);
        }
        coord_set
    }
}

impl CoordSet {
    /// The number of elements in the set.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let cs = CoordSet::from_iter(vec![(1,1),(1,1),(2,2)]);
    /// assert_eq!(cs.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.map(SetBits::count_ones).iter().sum::<u32>() as usize
    }

    /// Whether the set is empty.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::Coord;
    /// # use qsolve::datastructure::CoordSet;
    /// let cs = CoordSet::from_iter(vec![(1,1),(1,1),(2,2)]);
    /// assert!(!cs.is_empty());
    ///
    /// let cs2 = CoordSet::from_iter::<Vec<Coord>>(vec![]);
    /// assert!(cs2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }

    /// Tests whether the set contains a given coord.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let cs = CoordSet::from_iter(vec![(1,1),(1,1),(2,2)]);
    /// assert!(cs.contains(&(1,1)));
    /// assert!(!cs.contains(&(1,3)));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when either component of `coord` is outside `0..MAX_SET_SIZE`.
    pub fn contains(&self, coord: &Coord) -> bool {
        assert_coord_in_bounds(*coord);
        ((self.0[coord.0] >> (coord.1)) & 1) == 1
    }

    /// Adds a given coord to the set.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let mut cs = CoordSet::from_iter(vec![(1,1)]);
    /// assert_eq!(cs.len(), 1);
    /// assert!(!cs.contains(&(1,3)));
    /// cs.add((1,3));
    /// assert_eq!(cs.len(), 2);
    /// assert!(cs.contains(&(1,3)));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics when either component of `c` is outside `0..MAX_SET_SIZE`.
    pub fn add(&mut self, c: Coord) {
        assert_coord_in_bounds(c);
        self.0[c.0] |= 1 << c.1;
    }

    /// Efficiently computes the union between two CoordSets.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let cs1 = CoordSet::from_iter(vec![(1,1), (2,2), (3,3), (4,4)]);
    /// let cs2 = CoordSet::from_iter(vec![(3,3), (4,4), (5,5), (6,6)]);
    /// let union = cs1.union(&cs2);
    /// assert_eq!(union, CoordSet::from_iter(vec![(1,1), (2,2), (3,3), (4,4), (5,5), (6,6)]))
    /// ```
    pub fn union<'a>(&'a self, other: &'a CoordSet) -> CoordSet {
        let mut new_set = CoordSet::default();
        for a in 0..MAX_SET_SIZE {
            new_set.0[a] = self.0[a] | other.0[a];
        }
        new_set
    }

    /// Efficiently computes the intersection between two CoordSets.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let cs1 = CoordSet::from_iter(vec![(1,1), (2,2), (3,3), (4,4)]);
    /// let cs2 = CoordSet::from_iter(vec![(3,3), (4,4), (5,5), (6,6)]);
    /// let isect = cs1.intersection(&cs2);
    /// assert_eq!(isect, CoordSet::from_iter(vec![(3,3), (4,4)]))
    /// ```
    pub fn intersection<'a>(&'a self, other: &'a CoordSet) -> CoordSet {
        let mut new_set = CoordSet::default();
        for a in 0..MAX_SET_SIZE {
            new_set.0[a] = self.0[a] & other.0[a];
        }
        new_set
    }

    /// Returns an [Iterator] over the CoordSet.
    ///
    /// # Examples
    /// ```
    /// # use qsolve::datastructure::CoordSet;
    /// let cs = CoordSet::from_iter(vec![(1,1), (2,2), (3,3), (4,4)]);
    /// let smallsum = cs.iter().filter(|&(a,b)| a+b <= 4).collect::<CoordSet>();
    /// assert_eq!(smallsum, CoordSet::from_iter(vec![(1,1), (2,2)]))
    /// ```
    pub fn iter(&self) -> CoordSetIter<'_> {
        CoordSetIter {
            coord_set: self,
            idx: 0,
        }
    }
}

impl Extend<Coord> for CoordSet {
    fn extend<T: IntoIterator<Item = Coord>>(&mut self, iter: T) {
        for elem in iter {
            self.add(elem);
        }
    }
}

/// An iterator over [CoordSet].
pub struct CoordSetIter<'a> {
    coord_set: &'a CoordSet,
    idx: usize,
}

impl<'a> IntoIterator for &'a CoordSet {
    type Item = Coord;

    type IntoIter = CoordSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CoordSetIter {
            coord_set: self,
            idx: 0,
        }
    }
}

impl Iterator for CoordSetIter<'_> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < MAX_SET_SIZE * MAX_SET_SIZE {
            let a = self.idx / MAX_SET_SIZE;
            let b = self.idx % MAX_SET_SIZE;
            if ((self.coord_set.0[a] >> (b)) & 1) == 1 {
                self.idx += 1;
                return Some((a, b));
            }
            self.idx += 1;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(MAX_SET_SIZE * MAX_SET_SIZE - self.idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_color_set() {
        let sqs = SquareColorSet::from_iter([
            SquareColor::Black,
            SquareColor::White,
            SquareColor::Black,
            SquareColor::Blue,
        ]);
        assert_eq!(sqs.len(), 3);
        assert!(!sqs.is_empty());
        assert!(sqs.contains(&SquareColor::Black));
        assert!(!sqs.contains(&SquareColor::Red));
        assert_eq!(format!("{sqs}"), "[Black, Blue, White]");
    }

    #[test]
    fn line_set() {
        let ls = LineSet::from_iter([0, 2, 0, 5]);
        assert_eq!(ls.len(), 3);
        assert!(!ls.is_empty());
        assert!(ls.contains(&0));
        assert!(!ls.contains(&1));
        assert_eq!(ls.iter().collect::<Vec<_>>(), vec![0, 2, 5]);
        assert_eq!(format!("{ls}"), "[0, 2, 5]");
    }

    #[test]
    #[should_panic(expected = "is out of range for LineSet")]
    fn line_set_rejects_out_of_range_values() {
        let _ = LineSet::from_iter([MAX_SET_SIZE]);
    }

    #[test]
    #[should_panic(expected = "is out of range for LineSet")]
    fn line_set_rejects_out_of_range_queries() {
        LineSet::default().contains(&MAX_SET_SIZE);
    }

    #[test]
    fn coord_set() {
        let mut cs = CoordSet::from_iter([(0, 0), (1, 1), (0, 0), (2, 4)]);
        assert_eq!(cs.len(), 3);
        assert!(!cs.is_empty());
        assert!(cs.contains(&(0, 0)));
        assert!(!cs.contains(&(5, 5)));
        assert_eq!(cs.iter().collect::<Vec<_>>(), vec![(0, 0), (1, 1), (2, 4)]);
        assert_eq!(format!("{cs}"), "[(0, 0), (1, 1), (2, 4)]");
        cs.extend([(5, 5)]);
        assert!(cs.contains(&(5, 5)));

        let other = CoordSet::from_iter([(2, 4), (6, 6)]);
        assert_eq!(
            cs.union(&other),
            CoordSet::from_iter([(0, 0), (1, 1), (2, 4), (5, 5), (6, 6)])
        );
    }

    #[test]
    #[should_panic(expected = "is out of range for CoordSet")]
    fn coord_set_rejects_out_of_range_values() {
        let _ = CoordSet::from_iter([(MAX_SET_SIZE, 0)]);
    }

    #[test]
    #[should_panic(expected = "is out of range for CoordSet")]
    fn coord_set_rejects_out_of_range_queries() {
        CoordSet::default().contains(&(0, MAX_SET_SIZE));
    }

    #[test]
    #[should_panic(expected = "is out of range for CoordSet")]
    fn coord_set_rejects_out_of_range_additions() {
        CoordSet::default().add((MAX_SET_SIZE, 0));
    }
}
