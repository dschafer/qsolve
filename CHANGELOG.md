# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `cli` and `image` Cargo features, enabled by default, so library users can
  exclude the CLI or image support and their dependencies.
- Add Windows support by replacing `termion` with `crossterm`.
- Add a heuristic for pairs of rows or columns that eliminates squares ruled
  out by every legal queen pairing, allowing puzzles such as
  [#15](https://github.com/dschafer/qsolve/issues/15) to be solved without
  forcing chains.

### Fixed

- Give out-of-range values passed to bitset-backed sets consistent, descriptive panics.
- Return an error instead of panicking when an image contains no detectable grid.
- Calculate queen and X pixel ratios against the inspected image area.
- Preserve spaces used as blank markers at the edges of partial-state grids.

## [1.0.0] - 2025-04-23

### Added

- Initial release.

[Unreleased]: https://github.com/dschafer/qsolve/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/dschafer/qsolve/releases/tag/v1.0.0
