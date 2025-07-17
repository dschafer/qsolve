use std::fmt::{Display, Formatter};

use anyhow::{Result, bail};
use owo_colors::AnsiColors;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Represents a color that can appear on the board.
///
/// There are 16 possible colors, corresponding to the 16 ANSI colors
/// that most termianls support. The most critical traits and methods implemented on this
/// enum allow for easy converstion to and from different representations:
///
/// * `TryFrom<char>` for [SquareColor] allows for conversion from a [char] representation to a [SquareColor]. This is most notably used in parsing a [Board][crate::board::Board].
/// * `From<SquareColor>` for [AnsiColors] allows for conversion from a [SquareColor] to an [AnsiColors] for printing squares to the terminal.
/// * [SquareColor::to_unicode_block] returns the unicode block character for the color. This is useful for generating the share text for a solution.
///
/// Each color is represented by a [char]. The [char] is the first character of the color
/// for all colors other than black; for black, we use 'k' (since 'b' is already used for blue,
/// and 'k' matches the CMYK tradition).Normal colors are lower-cased, and bright colors are
/// upper-cased (so "Bright Red" would be 'R').
pub enum SquareColor {
    /// The Black ANSI color, represented by [char] 'k'.
    Black,
    /// The Red ANSI color, represented by [char] 'r'.
    Red,
    /// The Green ANSI color, represented by [char] 'g'.
    Green,
    /// The Yellow ANSI color, represented by [char] 'y'.
    Yellow,
    /// The Blue ANSI color, represented by [char] 'b'.
    Blue,
    /// The Magenta ANSI color, represented by [char] 'k'.
    Magenta,
    /// The Cyan ANSI color, represented by [char] 'c'.
    Cyan,
    /// The White ANSI color, represented by [char] 'w'.
    White,
    /// The Bright Black ANSI color, represented by [char] 'K'.
    BrightBlack,
    /// The Bright Red ANSI color, represented by [char] 'R'.
    BrightRed,
    /// The Bright Green ANSI color, represented by [char] 'G'.
    BrightGreen,
    /// The Bright Yellow ANSI color, represented by [char] 'Y'.
    BrightYellow,
    /// The Bright Blue ANSI color, represented by [char] 'B'.
    BrightBlue,
    /// The Bright Magenta ANSI color, represented by [char] 'K'.
    BrightMagenta,
    /// The Bright Cyan ANSI color, represented by [char] 'C'.
    BrightCyan,
    /// The Bright White ANSI color, represented by [char] 'W'.
    BrightWhite,
}

/// A convinience const that contains all square colors once.
pub const ALL_SQUARE_COLORS: [SquareColor; 16] = [
    SquareColor::Black,
    SquareColor::Red,
    SquareColor::Green,
    SquareColor::Yellow,
    SquareColor::Blue,
    SquareColor::Magenta,
    SquareColor::Cyan,
    SquareColor::White,
    SquareColor::BrightBlack,
    SquareColor::BrightRed,
    SquareColor::BrightGreen,
    SquareColor::BrightYellow,
    SquareColor::BrightBlue,
    SquareColor::BrightMagenta,
    SquareColor::BrightCyan,
    SquareColor::BrightWhite,
];

impl TryFrom<char> for SquareColor {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'k' => Ok(SquareColor::Black),
            'r' => Ok(SquareColor::Red),
            'g' => Ok(SquareColor::Green),
            'y' => Ok(SquareColor::Yellow),
            'b' => Ok(SquareColor::Blue),
            'm' => Ok(SquareColor::Magenta),
            'c' => Ok(SquareColor::Cyan),
            'w' => Ok(SquareColor::White),
            'K' => Ok(SquareColor::BrightBlack),
            'R' => Ok(SquareColor::BrightRed),
            'G' => Ok(SquareColor::BrightGreen),
            'Y' => Ok(SquareColor::BrightYellow),
            'B' => Ok(SquareColor::BrightBlue),
            'M' => Ok(SquareColor::BrightMagenta),
            'C' => Ok(SquareColor::BrightCyan),
            'W' => Ok(SquareColor::BrightWhite),
            _ => bail!("Unknown color char: {}", value),
        }
    }
}

impl From<SquareColor> for AnsiColors {
    fn from(c: SquareColor) -> AnsiColors {
        match c {
            SquareColor::Black => AnsiColors::Black,
            SquareColor::Red => AnsiColors::Red,
            SquareColor::Green => AnsiColors::Green,
            SquareColor::Yellow => AnsiColors::Yellow,
            SquareColor::Blue => AnsiColors::Blue,
            SquareColor::Magenta => AnsiColors::Magenta,
            SquareColor::Cyan => AnsiColors::Cyan,
            SquareColor::White => AnsiColors::White,
            SquareColor::BrightBlack => AnsiColors::BrightBlack,
            SquareColor::BrightRed => AnsiColors::BrightRed,
            SquareColor::BrightGreen => AnsiColors::BrightGreen,
            SquareColor::BrightYellow => AnsiColors::BrightYellow,
            SquareColor::BrightBlue => AnsiColors::BrightBlue,
            SquareColor::BrightMagenta => AnsiColors::BrightMagenta,
            SquareColor::BrightCyan => AnsiColors::BrightCyan,
            SquareColor::BrightWhite => AnsiColors::BrightWhite,
        }
    }
}

impl Display for SquareColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            SquareColor::Black => 'k',
            SquareColor::Red => 'r',
            SquareColor::Green => 'g',
            SquareColor::Yellow => 'y',
            SquareColor::Blue => 'b',
            SquareColor::Magenta => 'm',
            SquareColor::Cyan => 'c',
            SquareColor::White => 'w',
            SquareColor::BrightBlack => 'K',
            SquareColor::BrightRed => 'R',
            SquareColor::BrightGreen => 'G',
            SquareColor::BrightYellow => 'Y',
            SquareColor::BrightBlue => 'B',
            SquareColor::BrightMagenta => 'M',
            SquareColor::BrightCyan => 'C',
            SquareColor::BrightWhite => 'W',
        };
        write!(f, "{c}")
    }
}

impl SquareColor {
    /// Returns an appropriate Unicode block for the given color
    pub fn to_unicode_block(&self) -> char {
        match self {
            SquareColor::Black => '\u{2B1B}',
            SquareColor::Red => '\u{1F7E5}',
            SquareColor::Green => '\u{1F7E9}',
            SquareColor::Yellow => '\u{1F7E8}',
            SquareColor::Blue => '\u{1F7E6}',
            SquareColor::Magenta => '\u{1F7EA}',
            SquareColor::Cyan => '\u{1F7E6}',
            SquareColor::White => '\u{2B1C}',
            SquareColor::BrightBlack => '\u{2B1B}',
            SquareColor::BrightRed => '\u{1F7E5}',
            SquareColor::BrightGreen => '\u{1F7E9}',
            SquareColor::BrightYellow => '\u{1F7E8}',
            SquareColor::BrightBlue => '\u{1F7E6}',
            SquareColor::BrightMagenta => '\u{1F7EA}',
            SquareColor::BrightCyan => '\u{1F7E6}',
            SquareColor::BrightWhite => '\u{2B1C}',
        }
    }

    /// This returns the ideal foreground color for the given square color.
    ///
    /// It will always be one of [AnsiColors::Black] and [AnsiColors::BrightWhite]
    pub fn fg_color(&self) -> AnsiColors {
        match self {
            SquareColor::Black => AnsiColors::BrightWhite,
            SquareColor::BrightBlack => AnsiColors::BrightWhite,
            _ => AnsiColors::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squarecolor_char_roundtrip() {
        for sc in ALL_SQUARE_COLORS {
            let c = format!("{sc}").chars().next().unwrap();
            assert_eq!(SquareColor::try_from(c).unwrap(), sc)
        }
    }

    #[test]
    fn squarecolor_invalid_char() {
        assert!(SquareColor::try_from('E').is_err());
    }

    #[test]
    fn squarecolor_ansicolors() {
        for sc in ALL_SQUARE_COLORS {
            let ac = AnsiColors::from(sc);
            let fg = sc.fg_color();
            if ac == AnsiColors::Black || ac == AnsiColors::BrightBlack {
                assert_eq!(fg, AnsiColors::BrightWhite);
            } else {
                assert_eq!(fg, AnsiColors::Black);
            }
        }
    }

    #[test]
    fn squarecolor_unicode() {
        for sc in ALL_SQUARE_COLORS {
            let u = sc.to_unicode_block();
            assert!(!u.is_ascii())
        }
    }
}
