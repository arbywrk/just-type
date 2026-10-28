# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Terminal opening with the application on windows.

## [0.3.0] - 2026-10-27

### Changed

- Project renamed from type-game to just-type.

## [0.2.0] - 2026-10-27

### Added

- Custom widget for the typing text to enable properly adding a cursor.
- Ctrl + Backspace support for deleting an entire word.

### Fixed

- Invisible mistake on the space character. Now a mistake on space will show as an underline.

## [0.1.2] - 2026-10-19

### Added

- A new layout: the text sits in a fixed-width monospace block in the
  centre of the window, with the timer at its upper right.
- Only three lines of text are shown at a time, with at least one line always visible
  ahead of the one being typed.
- The results screen shows wpm, raw and accuracy as large numbers, plus the test length.
- Restart and Menu buttons on the results screen.

### Changed

- Restart generates a new text each time instead of reusing the previous one.
- Results are shown as whole numbers instead of two decimal places.

### Fixed

- The 15s option is selected on launch, matching the actual test length.
- Restart keeps the chosen duration instead of going back to the hard-coded default.
- The countdown is measured from the first key press, so it no longer drifts, and the
  game ends on time.
- Accuracy counts every key press, so mistakes corrected with backspace still count
  against it.
- Raw WPM includes characters that were later deleted.
- Enter, Tab, Escape, Delete and Ctrl shortcuts are no longer counted as wrong
  keystrokes or moving the position in the text.

## [0.1.1] - 2026-10-13

### Added

- Backspace support

## [0.1.0] - 2026-10-13

### Added

- Initial release.

[0.1.0]: https://github.com/arbywrk/just-type/releases/tag/v0.1.0
[0.1.1]: https://github.com/arbywrk/just-type/releases/tag/v0.1.1
[0.1.2]: https://github.com/arbywrk/just-type/releases/tag/v0.1.2
[0.2.0]: https://github.com/arbywrk/just-type/releases/tag/v0.2.0
[0.2.0]: https://github.com/arbywrk/just-type/releases/tag/v0.3.0
[Unreleased]: https://github.com/arbywrk/just-type/compare/v0.3.0...HEAD
