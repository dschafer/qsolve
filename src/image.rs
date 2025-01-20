use std::ops::Range;

use crate::board::Board;
use crate::file::{InputSquares, QueensFile};
use crate::solvestate::SquareVal;
use crate::squarecolor::{ALL_SQUARE_COLORS, SquareColor};

use anyhow::{Context, Result, anyhow, ensure};
use image::{GenericImageView, Rgb, RgbImage, SubImage};
use itertools::{Itertools, iproduct};
use log::trace;

/// RGB values for ANSI terminal colors
///
/// Taken from the VGA column of https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
const ANSI_COLORS: [(Rgb<u8>, SquareColor); 16] = [
    (Rgb([0, 0, 0]), SquareColor::Black),              // Black
    (Rgb([170, 0, 0]), SquareColor::Red),              // Red
    (Rgb([0, 170, 0]), SquareColor::Green),            // Green
    (Rgb([170, 85, 0]), SquareColor::Yellow),          // Yellow
    (Rgb([0, 0, 170]), SquareColor::Blue),             // Blue
    (Rgb([170, 0, 170]), SquareColor::Magenta),        // Magenta
    (Rgb([0, 170, 170]), SquareColor::Cyan),           // Cyan
    (Rgb([170, 170, 170]), SquareColor::White),        // White
    (Rgb([85, 85, 85]), SquareColor::BrightBlack),     // Bright Black (Gray)
    (Rgb([255, 85, 85]), SquareColor::BrightRed),      // Bright Red
    (Rgb([85, 255, 85]), SquareColor::BrightGreen),    // Bright Green
    (Rgb([255, 255, 85]), SquareColor::BrightYellow),  // Bright Yellow
    (Rgb([85, 85, 255]), SquareColor::BrightBlue),     // Bright Blue
    (Rgb([255, 85, 255]), SquareColor::BrightMagenta), // Bright Magenta
    (Rgb([85, 255, 255]), SquareColor::BrightCyan),    // Bright Cyan
    (Rgb([255, 255, 255]), SquareColor::BrightWhite),  // Bright White
];

/// Threshold value for determining if a pixel is considered black.
/// A pixel is considered black if all its RGB components are below this value.
const BLACK_THRESHOLD: u8 = 50;

/// The minimum ratio of black pixels required for a line to be considered a black grid line.
const BLACK_LINE_RATIO: f32 = 0.35;

/// The variance from the average black grid length allowed for a grid
/// to be considered a valid grid, as a percentage.
const GRID_LENGTH_VARIANCE: usize = 20;

/// The maximum number of colors to track in a given square.
/// Lowering this number saves a bit of memory, but risks detecting
/// "border" colors first and never finding the "true" dominant color.
const MAX_COLORS_TO_TRACK: usize = 1000;

/// The maximum thickness of a black grid line in the image. If this is too large, then the
/// algorithm might detect black borders around the image as a grid line.
const MAX_LINE_THICKNESS: usize = 20;

/// Maximum number of unique colors allowed in the grid
const MAX_UNIQUE_COLORS: usize = ALL_SQUARE_COLORS.len();

/// Threshold for determining if two colors are the same.
const COLOR_DISTANCE_THRESHOLD: u32 = 500;

/// Threshold for determining if a square contains a queen (high percentage of black pixels)
const QUEEN_OTHER_RATIO: f32 = 0.06;

/// Threshold for determining if a square contains an X (medium percentage of black pixels)
const X_OTHER_RATIO: f32 = 0.01;

/// Analyzes an image containing a grid of colored boxes and returns a [QueensFile].
///
/// # Arguments
/// * `img` - Reference to the [RgbImage] to analyze
///
/// # Returns
/// A [QueensFile] representing the grid of colored boxes with detected squares,
/// or [anyhow::Error] if the grid could not be constructed.
///
/// # Example
/// ```no_run
/// # use qsolve::image::analyze_grid_image;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let img = image::open("path/to/image.png")?.to_rgb8();
/// let queens_file = analyze_grid_image(&img)?;
/// # Ok(())
/// # }
/// ```
pub fn analyze_grid_image(img: &RgbImage) -> Result<QueensFile> {
    trace!("Analyze grid image start: {:?}", img);
    let width_ranges = find_grid_ranges(img, 0..img.width(), true);
    ensure!(
        width_ranges.len() >= 4,
        "Found too few columns; must be at least 4, found {}",
        width_ranges.len(),
    );
    let height_ranges = find_grid_ranges(img, 0..img.height(), false);
    ensure!(
        width_ranges.len() == height_ranges.len(),
        "Grid must be a square; width was {} height was {}",
        width_ranges.len(),
        height_ranges.len()
    );
    ensure!(
        width_ranges.len() <= MAX_UNIQUE_COLORS,
        "Grid is too large; max is {} found {}",
        MAX_UNIQUE_COLORS,
        width_ranges.len()
    );
    let board_size = width_ranges.len();

    let mut all_rgb_colors = Vec::with_capacity(board_size * board_size);
    let mut square_values = Vec::with_capacity(board_size * board_size);

    trace!(
        "Analyze grid image ranges found: {:?} {:?}",
        width_ranges, height_ranges
    );
    for (height_range, width_range) in iproduct!(height_ranges, width_ranges) {
        let view = img.view(
            width_range.start,
            height_range.start,
            width_range.end - width_range.start,
            height_range.end - height_range.start,
        );

        let rgb_color = get_dominant_color(&view).with_context(|| {
            format!(
                "Count not find dominant color in square at offset {:?}",
                view.offsets()
            )
        })?;
        trace!(
            "Analyze grid image color: for height {:?} width {:?} got dominant color {:?}",
            height_range, width_range, rgb_color
        );
        all_rgb_colors.push(rgb_color);

        let other_ratio = get_other_ratio(&view, &rgb_color);
        let square_val = match other_ratio {
            r if r >= QUEEN_OTHER_RATIO => Some(SquareVal::Queen),
            r if r >= X_OTHER_RATIO => Some(SquareVal::X),
            _ => None,
        };
        trace!(
            "Analyze grid image ratio: For height {:?} width {:?} got ratio {:?} and value {:?}",
            height_range, width_range, other_ratio, square_val
        );
        square_values.push(square_val);
    }

    let unique_rgb_colors = all_rgb_colors
        .clone()
        .into_iter()
        .unique()
        .collect::<Vec<_>>();

    ensure!(
        unique_rgb_colors.len() == board_size,
        "Number of unique colors must be equal to the board size"
    );

    // Map RGB colors to SquareColors
    let color_mapping = map_image_to_square_colors(&unique_rgb_colors);

    // Create the board colors
    let colors = all_rgb_colors
        .iter()
        .map(|&rgb_color| {
            let color_idx = unique_rgb_colors
                .iter()
                .position(|&c| c == rgb_color)
                .unwrap();
            color_mapping[color_idx]
        })
        .collect::<Vec<_>>();

    let board = Board::new(board_size, colors);
    let squares = InputSquares::from(square_values);
    trace!("Analyze grid image done.");
    trace!("Board:\n{}", board);
    trace!("Squares:\n{}", squares);
    Ok(QueensFile {
        board,
        squares: Some(squares),
    })
}

fn get_other_ratio(view: &SubImage<&RgbImage>, rgb_color: &Rgb<u8>) -> f32 {
    const BORDER_DENOM: u32 = 10;
    let (width, height) = view.dimensions();
    let center_subview = view.view(
        width / BORDER_DENOM,
        height / BORDER_DENOM,
        width - (2 * width / BORDER_DENOM),
        height - (2 * height / BORDER_DENOM),
    );
    let other_count = center_subview
        .pixels()
        .filter(|(_, _, p)| color_distance(*p, *rgb_color) > COLOR_DISTANCE_THRESHOLD)
        .count();
    (other_count as f32) / ((width * height) as f32)
}

fn find_grid_ranges(img: &RgbImage, range: Range<u32>, is_vertical: bool) -> Vec<Range<u32>> {
    // This is an optimization atop using ::collect(); we know that we're only going to find
    // at post MAX_UNIQUE_COLORS grid ranges, so we can allocate the vector with that capacity.
    let mut grid_ranges = Vec::with_capacity(MAX_UNIQUE_COLORS);
    let grid_ranges_iter = range
        .map(|x| (x, black_ratio(img, x, is_vertical) > BLACK_LINE_RATIO))
        .dedup_by_with_count(|(_, i), (_, j)| i == j)
        .filter(|&(count, (_, is_black))| is_black && count < MAX_LINE_THICKNESS)
        .tuple_windows()
        .map(|((start_count, (start_idx, _)), (_, (end_idx, _)))| {
            (start_idx + start_count as u32)..end_idx
        });
    grid_ranges.extend(grid_ranges_iter);
    let grid_ranges_len = grid_ranges.len();
    let median_grid_length = grid_ranges
        .clone()
        .select_nth_unstable_by(grid_ranges_len / 2, |a, b| a.len().cmp(&b.len()))
        .1
        .len();
    grid_ranges
        .into_iter()
        .filter(|r| {
            r.len() > (median_grid_length * (100 - GRID_LENGTH_VARIANCE)) / 100
                && r.len() < (median_grid_length * (100 + GRID_LENGTH_VARIANCE)) / 100
        })
        .collect::<Vec<_>>()
}

/// Helper function to check if a line (horizontal or vertical) is black
fn black_ratio(img: &RgbImage, pos: u32, is_vertical: bool) -> f32 {
    let total = if is_vertical {
        img.height()
    } else {
        img.width()
    };
    let black_count = (0..total)
        .map(|i| {
            if is_vertical {
                img.get_pixel(pos, i)
            } else {
                img.get_pixel(i, pos)
            }
        })
        .filter(|&pixel| is_black(pixel))
        .count();

    black_count as f32 / total as f32
}

/// Helper function to determine if a pixel is black
fn is_black(pixel: &Rgb<u8>) -> bool {
    pixel[0] < BLACK_THRESHOLD && pixel[1] < BLACK_THRESHOLD && pixel[2] < BLACK_THRESHOLD
}

/// Helper function to get the dominant color in a box
fn get_dominant_color(img: &SubImage<&RgbImage>) -> Result<Rgb<u8>> {
    let mut colors = [Rgb([0, 0, 0]); MAX_COLORS_TO_TRACK];
    let mut counts = [0u32; MAX_COLORS_TO_TRACK];
    let mut num_colors = 0;

    for pixel in img.pixels().map(|(_, _, p)| p).filter(|&p| !is_black(&p)) {
        match (
            num_colors,
            colors[..num_colors].iter().position(|&p| p == pixel),
        ) {
            (_, Some(idx)) => counts[idx] += 1,
            (MAX_COLORS_TO_TRACK, None) => (),
            (_, None) => {
                colors[num_colors] = pixel;
                counts[num_colors] = 1;
                num_colors += 1;
            }
        }
    }
    counts[..num_colors]
        .iter()
        .zip(colors[..num_colors].iter())
        .max_by(|&(a, _), &(b, _)| a.cmp(b))
        .map(|(_, color)| *color)
        .ok_or_else(|| anyhow!("Could not find dominant color"))
}

/// Calculates the color distance between two RGB values using the Euclidean distance
fn color_distance(rgb1: Rgb<u8>, rgb2: Rgb<u8>) -> u32 {
    ((rgb1[0] as u32).abs_diff(rgb2[0] as u32)).pow(2)
        + ((rgb1[1] as u32).abs_diff(rgb2[1] as u32)).pow(2)
        + ((rgb1[2] as u32).abs_diff(rgb2[2] as u32)).pow(2)
}

/// Maps image colors to SquareColors by trying all NxM combinations, assigning that color,
/// removing the matched colors from the set, and repeating
fn map_image_to_square_colors(image_colors: &[Rgb<u8>]) -> [SquareColor; MAX_UNIQUE_COLORS] {
    let mut image_to_square_color = [SquareColor::Black; MAX_UNIQUE_COLORS];
    let mut used_square_colors = [false; MAX_UNIQUE_COLORS];
    let mut used_image_colors = [false; MAX_UNIQUE_COLORS];

    for _ in 0..image_colors.len() {
        let mut min_distance = u32::MAX;
        let mut best_square_color_idx = 0;
        let mut best_image_color_idx = 0;
        for (image_color_idx, image_rgb) in image_colors.iter().enumerate() {
            if used_image_colors[image_color_idx] {
                continue;
            }
            for (square_color_idx, (square_rgb, _)) in ANSI_COLORS.iter().enumerate() {
                if used_square_colors[square_color_idx] {
                    continue;
                }
                let distance = color_distance(*image_rgb, *square_rgb);
                if distance < min_distance {
                    min_distance = distance;
                    best_square_color_idx = square_color_idx;
                    best_image_color_idx = image_color_idx;
                }
            }
        }

        image_to_square_color[best_image_color_idx] = ANSI_COLORS[best_square_color_idx].1;
        used_square_colors[best_square_color_idx] = true;
        used_image_colors[best_image_color_idx] = true;
    }

    image_to_square_color
}
