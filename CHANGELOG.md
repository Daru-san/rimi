## [0.1.6] - 2024-12-21

### Features

- Implement parallel image decoding with rayon
- Experiment with batch image saving and processing
- Obtain processed tasks
- Allow specifical of parallel task count
- Create command message in a separate function
- Add custom testing build profile
- Bump up possible loaded images to 10000

### Bug Fixes

- Set task count before running single mode
- Remove is_alpha from info display
- Reference to mutable image

### Refactor

- Iterator
- Single_progress -> progress
- Impl task count as part of AppProgress
- Create empty progress bars when --quiet flag is passed
- Fix typo
- Split ColorType into BitDepth and ColorTypeExt
- Do not implement copy trait for BitDepth
- Use new color info when converting images
- Name ColorTypeExt as ColorSpace
- Use 'b' bit depth flag over 'B'
- Remove PartialEq impl for BitDepth
- To_color_space() -> to_color_type()
- Remove unneeded variable
- Add trace for failed images
- Add trace
- Circle command args
- Use std::mem::take to obtain image files without cloning
- Pass output_paths as a reference instead of cloning
- Fix paramater
- Fix batch image references
- Use std::mem::take to obtain image files without cloning
- Pass output_paths as a reference instead of cloning
- Fix paramater
- Wrap tasks_queue in arc mutex
- Update references to tasks_queue to account for mutex lock
- Pass progress indicator as arc mutex
- Run command in it's own function
- Spawn threads in non-fifo order
- Remove unused queue function
- Rename parallel tasks flag
- Fix queue lock blocking when multiple threads are running
- Use iterators when filtering tasks
- Named threads in thread pool
- Only spawn new processing threads when needed
- Split image writing and processing into separate steps
- Yeat task from queue when completed
- Add working task state
- Add conversion function
- Add convert function to single mode
- Support in-memory image conversion
- Return when converting to avif
- Circle command args
- Use rayon par bridge when removing backgrounds
- Use take over clone
- Remove unused import
- Add trace
- Use load_from_memory over image reader
- Use parallel iterator to read files
- Use try_for_each to stop searching for tasks when they are found
- Remove unused import
- Store images and tasks as option types
- Prefer parallel iterators over a thread pool
- Remove Arcs
- Rename cursor to writer
- Save image using a buffered writer
- Refactor out path function
- Merge run_command and command_msg functions
- Pass mutex guards using try
- Remove unneeded progress functions
- Improve progress indicators
- Rename progress bar and tasks_queue
- Use for loops
- Remove unneeded returns
- Replace the tasks queue with an ordinary vector
- Experiment with raw buffer writing
- Remove unused code
- Add 1 to buffer capacity
- Drop failed tasks
- Fix a few grammar errors in path prompt
- Remove batch error from task error
- Reorganize task message spawning
- Remove unused flags
- Fix image formatting logic
- Use collect instead of pushing to a new vector when path checking
- Update header

### ⚙️ Miscellaneous Tasks

- Remove console
- Bump version to 0.1.5
- Update gitignore
- Add cargo-profiler to flake devshell
- Add cargo-profiler to flake devshell
- Update gitignore
- Update cargo deps
- Bump to 0.1.6
## [0.1.5-alpha] - 2024-12-16

### Features

- Initialize app state struct
- Create state function
- Expand command arguments
- Implement `run_single()` function
- Implement WIP `run_batch()` function
- Add command matching to batch
- Flesh out Task and TaskQueue
- Store out paths in decoded images
- Set tasks as complete
- Save image and push result to task queue
- Use indicatif and console to display progress
- Add add GlobalProgress for tracking singular image progres
- Split progress into single and batch
- Add error module
- Add imports
- Implement progress verbosity
- Check if paths exists and prompt in batch mode
- Add suspend function to progress
- Check for overwrite in single mode

### Bug Fixes

- Remove unused imports
- Catch errors when running app
- Check if directory is a file when creating paths
- Actually count failures when using count_failures()
- Clone task to prevent mutable borrow
- Yeat bracket
- Add missing bracket
- Import display module
- Import
- Make error module public
- Import
- Loop without count variable
- Publicize ExtraArgs
- Remove unnecessary borrow
- Function name
- Prevent double progress bars in batch mode
- Abandon subtask progress bar before task progress bar
- Include image module
- Join destination paths with file names, not absolute paths
- Remove overwrite code from image module

### Refactor

- Move arguments to command module
- Run CommandArgs, not Command enum
- Impl CommandArgs not Command
- Yeat batch module
- Derive CompletionArgs with Debug
- Derive argument structs with Debug
- Remove convert module
- Recolor does not save images, remove image flags
- Resize does not save images, remove path args
- Transparent does not save images
- Clean up arguments
- Imports on command.rs
- Use paths instead of strings with batch operations
- Use a path instead of a string for output of save_image_format()
- Remove unused app arguments
- Yeat check_batch()
- Do not check images before transparentizing
- Remove references to GlobalArgs
- Do not show unnecessary error when running single mode
- Comma
- Pass references to self,path and images
- Use new out_path when saving images in batch
- Fixup completions
- Fix conflicting arguments
- Globalize a few arguments
- Do not refer to image_path
- Show error when no images are selected
- Wrap TaskError in BatchError
- Do not check for format
- Send images to task queue when decoding
- Bad attempt at modifying items in the task queue
- Allow cloning of ImageTask,TaskState and TaskError
- Replace `match` patterns with `if let`
- Pass paths as path parameters, not pathbuf
- Move function up
- Remove unused import
- Convert image path to path buf instead of cloning
- Replace `match` with `if let`
- Using tasks_queue.set_..() instead of directly modifying tasks
- Remove dead code
- Skip current iteration of loop when image has failed
- Improve incorrect command error message
- Always take task from first index
- Remove redundant line
- Tasks info does not need to be public
- Move state module to backend
- Split runner into single and batch
- Remove run_single() and run_batch() from command module
- Tasks info does not need to be public
- Store errors in TaskError and AppError
- Replace Box<dyn Error> with anyhow::Result types
- Propagate errors using TaskError
- Propagate TaskErrors
- More task errors
- Propagate NoSuchTask error when task is not found
- Pass error to error_sub_operation without formatting
- Remove unnecessary .into()
- Split image and misc subcommands into separate structs
- Pass subcommand flags via Run traits
- Pass command and verbosity to run() functions
- Expand error names
- Pass paths as &Path and not PathBuf
- Pass image file as path in info command
- Improve task queue method names
- Improve variable names in image module
- Update task queue method names in batch.rs
- Rename progress methods to reflect tasks
- Abort progress with message when failed batch tasks
- Out with unused completed_tasks() function
- Rename progress bars
- Reflect renamed progress bars
- Complete subtask progress before task progress
- Set progress chars as '#'
- Improve module names
- Remove console output from run_single()
- Prevent --verbose and --quite being passed together
- Use match to check verbosity
- Abort progress bar on path creation failure
- Fix path not parsing due to file_name() return an option strign
- Show subtask errors on complete batch
- Replace .as_str() with & references
- Show message at the end of every decode
- Expand width and heigh parameters
- Split image runner into a struct
- Account for image formats when creating paths

### ⚙️ Miscellaneous Tasks

- Update CHANGELOG
- Dist does not create release
- Add console and indicatif
- Add flake-lock action
- Add auth token to flake lock action
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
