## [0.1.4] - 2024-12-12

### Features

- Bump to 0.1.3
- Bump to 0.1.4

## [0.1.3] - 2024-12-10

### Refactor

- Fixup completions

### ⚙️ Miscellaneous Tasks

- Fix changelog generator action path

## [0.1.2] - 2024-12-07

### Features

- Use git-cliff to generate changelogs
- Prompt when overwriting an existing file
- Add `-i` flag to show image information
- Add `--preserve_aspect` flag when resizing
- Initialize batch image checking
- Open images with error checking using open_image()
- Add batch saving
- Add background removal
- Add filesize and file type to info
- Implement ColorType
- Add recolor flag
- Add shell completions
- Add shell completions to nix package
- Add conversion struct
- Add transparency functions
- Re-implement recolor functionality
- Init shell completions
- Add commands enum
- Implement app args
- Simplify main completely
- Bump to 0.1.2

### Bug Fixes

- Add newline
- Defer image when calling image.resize()
- Switch width and height on print_info()
- Add missing import
- Pass path as pathbuf to print_info()
- Remove unused imports
- Cleanup unused imports
- Import util modules
- Fix references to app::GlobalArgs from app::Args

### Refactor

- Remove save all images with formatting
- Add a descriptive message when no commands are selected
- Open image using ImageReader::open()
- Pass image format as Option<&str>
- Fix typo
- Set image path and directory as command-specific arguments
- Rename resize arguments to width and height
- Simplify image format
- Get format from path
- Check bit depth when removing background
- Ensure image is a png when removing background
- Print help when no option is selected
- Simply color matching
- Make FromStr for ColorInfo more readable
- Luminant is not a color type, rename to Luma
- Split batch module
- Clean utils module
- Rewrite open_image() with error returning
- Handle errors with save_image_format()
- Implement remaining image functions
- Fix up batch module
- Re-implement resize module
- Re-implement info functions
- Move color module to utils
- Refer to new color module in image and info modules
- Remove `open` module
- Use iterator index to get path from vector
- Increase image index by one, not two
- Overhaul image manipulation error handling
- Improve error handling when doing batch operations
- Improve error handling when checking for overwrites
- Check with lowercase when getting color info
- Set batch output directory flag as pathbuf

### ⚙️ Miscellaneous Tasks

- Add gif-cliff and cargo-dist to shell env
- Add nix commit formats to git-cliff
- Add new contributors to git-cliff changelog
- Generate changelogs on release
- Update subcommand docs
- Add omitted export
- Update cargo deps
- Add clap_complete_nushell to dependencies
- Update dependencies

## [0.1.0] - 2024-11-21

### ⚙️ Miscellaneous Tasks

- Update cargo.lock

